use std::future::Future;
use std::sync::Arc;

use dioxus::core::ScopeState;
use futures_channel::mpsc;

/// Hook that runs futures with input from [run](UseFuture::run) to completion, triggering an update of the component
/// this hook belongs to when the future completes, providing the values those futures produced through
/// [try_take](UseFuture::iter_take).
pub struct UseFuture<I, O> {
  input_tx: mpsc::Sender<I>,
  input_rx: mpsc::Receiver<I>,
  output_tx: mpsc::Sender<O>,
  output_rx: mpsc::Receiver<O>,
  update: Arc<dyn Fn()>,
}

/// Extension trait for using [future hooks](UseFuture).
pub trait UseFutureExt<I, O> {
  /// Uses a [future hook](UseFuture) on the component of `self`, creating channels with `channel_capacity`, using
  /// `create_future` to create futures with inputs from [run](UseFuture::run), and run them to completion.
  fn use_future<F: Future<Output=O> + 'static>(
    &self,
    channel_capacity: usize,
    create_future: impl FnMut(I) -> F
  ) -> &mut UseFuture<I, O>;
}
impl<I: 'static, O: 'static> UseFutureExt<I, O> for ScopeState {
  #[inline]
  fn use_future<F: Future<Output=O> + 'static>(
    &self,
    channel_capacity: usize,
    mut create_future: impl FnMut(I) -> F
  ) -> &mut UseFuture<I, O> {
    let use_future = self.use_hook(move || {
      let (input_tx, input_rx) = mpsc::channel::<I>(channel_capacity);
      let (output_tx, output_rx) = mpsc::channel::<O>(channel_capacity);
      UseFuture { input_tx, input_rx, output_tx, output_rx, update: self.schedule_update() }
    });

    // Ignore error OK: not a problem if there are no messages but the channel is not yet closed.
    for input in std::iter::from_fn(|| use_future.input_rx.try_next().ok().flatten()) {
      let future = (create_future)(input);
      let mut tx = use_future.output_tx.clone();
      let update = use_future.update.clone();
      self.push_future(async move {
        let value = future.await;
        let _ = tx.try_send(value); // TODO: should not ignore the error when it is full?
        (update)();
      });
    }

    use_future
  }
}

impl<I, O: 'static> UseFuture<I, O> {
  /// Run a future with `input` to completion the next time this hook is used. Triggers an update of the component this
  /// hook belongs to.
  #[inline]
  pub fn run(&self, input: I) {
    let _ = self.input_tx.clone().try_send(input); // TODO: should not ignore the error when it is full?
    (self.update)();
  }

  /// Iterates over all values produced by completed futures and takes them.
  ///
  /// This method takes the values out, so it will only return them once.
  #[inline]
  pub fn iter_take(&mut self) -> impl Iterator<Item=O> + '_ {
    // Ignore error OK: not a problem if there are no messages but the channel is not yet closed.
    std::iter::from_fn(|| self.output_rx.try_next().ok().flatten())
  }
}

/// Handle for running futures with a [future hook](UseFuture). Can be [cloned](Clone).
#[derive(Clone)]
pub struct UseFutureRunHandle<I> {
  tx: mpsc::Sender<I>,
  update: Arc<dyn Fn()>,
}
impl<I, O: 'static> UseFuture<I, O> {
  /// Creates a [future hook run handle](UseFutureRunHandle) for running futures, but which can also be [cloned](Clone).
  #[inline]
  pub fn run_handle(&self) -> UseFutureRunHandle<I> {
    UseFutureRunHandle { tx: self.input_tx.clone(), update: self.update.clone() }
  }
}
impl<I> UseFutureRunHandle<I> {
  /// Run a future with `input` to completion the next time the hook of this handle is used. Triggers an update of the
  /// component the hook of this handle belongs to.
  #[inline]
  pub fn run(&self, input: I) {
    let _ = self.tx.clone().try_send(input); // TODO: should not ignore the error when it is full?
    (self.update)();
  }
}
