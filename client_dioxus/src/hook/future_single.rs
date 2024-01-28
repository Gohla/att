use std::future::Future;
use std::sync::Arc;

use dioxus::core::{ScopeState, TaskId};
use futures_channel::oneshot;

/// Hook that runs a future to completion when [run](UseFutureSingle::run) is called, triggering an update of the component
/// this hook belongs to when the future completes, providing the value that future produced through
/// [try_take](UseFutureSingle::try_take).
///
/// Only stores the single value produced by the future that completed last.
pub struct UseFutureSingle<T> {
  update: Arc<dyn Fn()>,
  should_run: bool,
  running: Option<Running<T>>,
}
struct Running<T> {
  rx: oneshot::Receiver<T>,
  task: TaskId,
}

/// Extension trait for using [future single hooks](UseFutureSingle).
pub trait UseFutureSingleExt<T> {
  /// Uses a [future single hook](UseFutureSingle) on the component of `self`, using `create_future` to create a future
  /// and run it to completion when [run](UseFutureSingle::run) has been called previously.
  fn use_future_single<F: Future<Output=T> + 'static>(&self, create_future: impl FnOnce() -> F) -> &mut UseFutureSingle<T>;
}
impl<T: 'static> UseFutureSingleExt<T> for ScopeState {
  #[inline]
  fn use_future_single<F: Future<Output=T> + 'static>(
    &self,
    create_future: impl FnOnce() -> F
  ) -> &mut UseFutureSingle<T> {
    let use_future_single = self.use_hook(move ||
      UseFutureSingle { update: self.schedule_update(), should_run: false, running: None }
    );
    if use_future_single.should_run {
      if let Some(running) = use_future_single.running.take() {
        self.remove_future(running.task);
      }

      let (tx, rx) = oneshot::channel::<T>();
      let future = create_future();
      let update = use_future_single.update.clone();
      let task = self.push_future(async move {
        if let Ok(()) = tx.send(future.await) {
          update();
        }
      });

      use_future_single.should_run = false;
      use_future_single.running = Some(Running { rx, task });
    }
    use_future_single
  }
}

impl<T> UseFutureSingle<T> {
  /// Run a future to completion the next time this hook is used. If a future was already running, it is cancelled and
  /// its produced value will be ignored. Triggers an update of the component this hook belongs to.
  #[inline]
  pub fn run(&mut self) {
    (self.update)();
    self.should_run = true;
  }

  /// Tries to take the value created by the future, returning `Some(value)` if the future has completed, or `None` if
  /// the future is pending or cancelled.
  ///
  /// This method takes the value out, so it will return `None` after this method has returned `Some(value)`, until
  /// [run](Self::run) is called to run a future again.
  #[inline]
  pub fn try_take(&mut self) -> Option<T> {
    self.running.as_mut().and_then(|r| r.rx.try_recv().ok().flatten())
  }
}
