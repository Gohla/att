use std::future::Future;
use std::sync::Arc;

use dioxus::core::{ScopeState, TaskId};
use futures_channel::oneshot;

/// Hook that runs a future to completion when [run](UseFuture::run) is called, triggering an update of the component
/// this hook belongs to when the future completes, providing the value that future produced through
/// [try_take](UseFuture::try_take).
pub struct UseFuture<T> {
  update: Arc<dyn Fn()>,
  should_run: bool,
  running: Option<Running<T>>,
}
struct Running<T> {
  rx: oneshot::Receiver<T>,
  task: TaskId,
}

/// Extension trait for using [future hooks](UseFuture).
pub trait UseFutureExt<T> {
  /// Uses a [future hook](UseFuture) on the component of `self`, using `create_future` to create a future and
  /// run it to completion when [run](UseFuture::run) was called before.
  fn use_future<F: Future<Output=T> + 'static>(&self, create_future: impl FnOnce() -> F) -> &mut UseFuture<T>;
}
impl<T: 'static> UseFutureExt<T> for ScopeState {
  #[inline]
  fn use_future<F: Future<Output=T> + 'static>(
    &self,
    create_future: impl FnOnce() -> F
  ) -> &mut UseFuture<T> {
    let use_future = self.use_hook(move ||
      UseFuture { update: self.schedule_update(), should_run: false, running: None }
    );
    if use_future.should_run {
      if let Some(running) = use_future.running.take() {
        self.remove_future(running.task);
      }

      let (tx, rx) = oneshot::channel::<T>();
      let future = create_future();
      let update = use_future.update.clone();
      let task = self.push_future(async move {
        if let Ok(()) = tx.send(future.await) {
          update();
        }
      });

      use_future.should_run = false;
      use_future.running = Some(Running { rx, task });
    }
    use_future
  }
}

impl<T> UseFuture<T> {
  /// Run a future to completion the next time this hook is used. If a future was already running, it is cancelled and
  /// its produced value will be ignored. Triggers an update of the component this hook belongs to.
  #[inline]
  pub fn run(&mut self) {
    (self.update)();
    self.should_run = true;
  }

  /// Tries to take the value created by the future if completed, returning `Some(value)` if the future has completed,
  /// or `None` if the future is pending, cancelled.
  #[inline]
  pub fn try_take(&mut self) -> Option<T> {
    self.running.as_mut().and_then(|r| r.rx.try_recv().ok().flatten())
  }
}
