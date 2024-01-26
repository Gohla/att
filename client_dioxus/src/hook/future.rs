use std::future::Future;

use dioxus::core::ScopeState;
use futures_channel::oneshot;

/// Hook that runs a future once, triggering an update of the component this hook belongs to when the future completes.
pub struct UseFutureOnce<T> {
  rx: oneshot::Receiver<T>,
}
impl<T: 'static> UseFutureOnce<T> {
  /// Creates a [once future hook](UseFutureOnce) on the component of `cx`, using `create_future` to create and then run
  /// that future.
  #[inline]
  pub fn hook<F: Future<Output=T> + 'static>(
    cx: &ScopeState,
    create_future: impl FnOnce() -> F,
  ) -> &mut UseFutureOnce<T> {
    cx.use_hook(move || {
      let (tx, rx) = oneshot::channel::<T>();
      let future = create_future();
      let update = cx.schedule_update();
      cx.push_future(async move {
        if let Ok(()) = tx.send(future.await) {
          update();
        }
      });
      UseFutureOnce { rx }
    })
  }
}
pub trait UseFutureOnceExt<T> {
  /// Creates a [once future hook](UseFutureOnce) on the component of `self`, using `create_future` to create and then
  /// run that future.
  fn use_future_once<F: Future<Output=T> + 'static>(
    &self,
    create_future: impl FnOnce() -> F
  ) -> &mut UseFutureOnce<T>;
}
impl<T: 'static> UseFutureOnceExt<T> for ScopeState {
  #[inline]
  fn use_future_once<F: Future<Output=T> + 'static>(
    &self,
    create_future: impl FnOnce() -> F
  ) -> &mut UseFutureOnce<T> {
    UseFutureOnce::hook(self, create_future)
  }
}

impl<T> UseFutureOnce<T> {
  /// Gets the value created by the future if completed, returning `Some(value)` if the future has completed, or `None`
  /// if the future is pending or cancelled.
  #[inline]
  pub fn get(&mut self) -> Option<T> {
    self.rx.try_recv().ok().flatten()
  }
}
