use std::future::Future;

use dioxus::core::ScopeState;
use futures_channel::oneshot;

/// Hook that immediately runs a future to completion once, triggering an update of the component this hook belongs to
/// when the future completes, providing the value that future produced through [try_take](UseFutureOnce::try_take).
pub struct UseFutureOnce<T> {
  rx: oneshot::Receiver<T>,
}

/// Extension trait for using [once future hooks](UseFutureOnce).
pub trait UseFutureOnceExt<T> {
  /// Uses a [once future hook](UseFutureOnce) on the component of `self`, using `create_future` to create a future and
  /// run it to completion once.
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
    self.use_hook(move || {
      let (tx, rx) = oneshot::channel::<T>();
      let future = create_future();
      let update = self.schedule_update();
      self.push_future(async move {
        if let Ok(()) = tx.send(future.await) {
          update();
        }
      });
      UseFutureOnce { rx }
    })
  }
}

impl<T> UseFutureOnce<T> {
  /// Tries to take the value created by the future if completed, returning `Some(value)` if the future has completed,
  /// or `None` if the future is pending, cancelled, or if this method already returned `Some(value)` before.
  #[inline]
  pub fn try_take(&mut self) -> Option<T> {
    self.rx.try_recv().ok().flatten()
  }
}
