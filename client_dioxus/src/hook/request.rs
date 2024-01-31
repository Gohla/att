use std::future::Future;
use std::sync::Arc;

use dioxus::core::ScopeState;
use futures::channel::mpsc;

/// Hook for sending requests of type `Q`, processing pending requests via futures that produce a response of type `S`.
pub struct UseRequest<Q, S> {
  request_tx: RequestSender<Q>,
  request_rx: mpsc::Receiver<Q>,
  response_tx: mpsc::Sender<S>,
  response_rx: ResponseReceiver<S>,
}

/// Extension trait for using [request hooks](UseRequest).
pub trait UseRequestExt<Q, S> {
  /// Uses a [request hook](UseRequest) on the component of `self`, creating channels with `channel_capacity` the first
  /// time this hook is used.
  ///
  /// Futures are (optionally) created with `create_future_for_request` for all pending requests, and ran to completion
  /// in the background.
  ///
  /// Returns a request sender and a response receiver.
  fn use_request_opt<F: Future<Output=S> + 'static>(
    &self,
    channel_capacity: usize,
    create_future_for_request: impl FnMut(Q) -> Option<F>,
  ) -> (&RequestSender<Q>, &mut ResponseReceiver<S>);
  /// Uses a [request hook](UseRequest) on the component of `self`, creating channels with `channel_capacity` the first
  /// time this hook is used.
  ///
  /// Futures are (optionally) created with `create_future_for_request` for all pending requests, and ran to completion
  /// in the background.
  ///
  /// Returns a request sender and a response receiver.
  #[inline]
  fn use_request<F: Future<Output=S> + 'static>(
    &self,
    channel_capacity: usize,
    mut create_future_for_request: impl FnMut(Q) -> F,
  ) -> (&RequestSender<Q>, &mut ResponseReceiver<S>) {
    self.use_request_opt(channel_capacity, |input| Some(create_future_for_request(input)))
  }
}
impl<Q: 'static, S: 'static> UseRequestExt<Q, S> for ScopeState {
  #[inline]
  fn use_request_opt<F: Future<Output=S> + 'static>(
    &self,
    channel_capacity: usize,
    mut create_future_for_request: impl FnMut(Q) -> Option<F>
  ) -> (&RequestSender<Q>, &mut ResponseReceiver<S>) {
    let use_request = self.use_hook(move || {
      let (request_tx, request_rx) = mpsc::channel::<Q>(channel_capacity);
      let (response_tx, response_rx) = mpsc::channel::<S>(channel_capacity);
      let request_tx = RequestSender { tx: request_tx, update: self.schedule_update() };
      let response_rx = ResponseReceiver { rx: response_rx };
      UseRequest { request_tx, request_rx, response_rx, response_tx }
    });

    // Ignore error OK: not a problem if there are no messages but the channel is not yet closed.
    for input in std::iter::from_fn(|| use_request.request_rx.try_next().ok().flatten()) {
      if let Some(future) = create_future_for_request(input) {
        let mut tx = use_request.response_tx.clone();
        let update = use_request.request_tx.update.clone();
        self.push_future(async move {
          let value = future.await;
          let _ = tx.try_send(value); // TODO: should not ignore the error when it is full?
          update();
        });
      }
    }

    (&use_request.request_tx, &mut use_request.response_rx)
  }
}

/// [Cloneable](Clone) request sender.
#[derive(Clone)]
pub struct RequestSender<Q> {
  tx: mpsc::Sender<Q>,
  update: Arc<dyn Fn()>,
}
impl<Q> RequestSender<Q> {
  /// Sends `request` the next time the hook of this handle is used. Triggers an update of the component the hook of
  /// this handle belongs to.
  #[inline]
  pub fn send(&self, request: Q) {
    let _ = self.tx.clone().try_send(request); // TODO: should not ignore the error when it is full?
    (self.update)();
  }
}

/// Response receiver.
pub struct ResponseReceiver<S> {
  rx: mpsc::Receiver<S>,
}
impl<S> ResponseReceiver<S> {
  /// Drains all received responses.
  ///
  /// This method drains (takes out) the responses, so the responses will only be returned once.
  #[inline]
  pub fn drain(&mut self) -> impl Iterator<Item=S> + '_ {
    // Ignore error OK: not a problem if there are no messages but the channel is not yet closed.
    std::iter::from_fn(|| self.rx.try_next().ok().flatten())
  }
}

