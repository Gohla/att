use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use att_core::query::{Query, QueryMessage};
use att_core::util::future::OptFutureExt;
use att_core::util::maybe_send::{MaybeSend, MaybeSendOptFuture};
use att_core::util::time::{Instant, sleep};

#[derive(Debug)]
pub struct QuerySender<Q: Query> {
  query: Q,
  query_config: Q::Config,
  wait_until: Option<Instant>,
  wait_duration: Duration,
  send_query_if_empty: bool,
}
impl<Q: Query> QuerySender<Q> {
  pub fn new(query: Q, query_config: Q::Config, wait_duration: Duration, send_query_if_empty: bool) -> Self {
    Self {
      query,
      query_config,
      wait_until: None,
      wait_duration,
      send_query_if_empty,
    }
  }

  /// Returns the query.
  #[inline]
  pub fn query(&self) -> &Q { &self.query }

  /// Returns the query.
  #[inline]
  pub fn query_config(&self) -> &Q::Config { &self.query_config }
}

// Send specific requests

impl<Q: Query> QuerySender<Q> {
  /// Update the query from `message`, returning a future producing a [response](WaitCleared) that must be
  /// [processed](Self::process_wait_cleared).
  ///
  /// If `send_query_if_empty` is `false` and the query is empty: returns `None` and any ongoing query will not be sent
  /// when the wait is cleared.
  pub fn update_query(&mut self, message: QueryMessage) -> Option<impl Future<Output=WaitCleared>> {
    message.update_query(&mut self.query, &self.query_config);
    if !self.send_query_if_empty && self.query.is_empty(&self.query_config) {
      self.wait_until = None;
      None
    } else {
      let wait_until = Instant::now() + self.wait_duration;
      self.wait_until = Some(wait_until);
      let future = sleep(self.wait_duration);
      let future = async move {
        future.await;
        WaitCleared
      };
      Some(future)
    }
  }
}

// Process specific responses

/// Wait cleared response.
#[derive(Debug)]
pub struct WaitCleared;

impl<Q: Query + Clone> QuerySender<Q> {
  /// Process a wait cleared response, returning `Some(query)` if the query should be sent, `None` otherwise.
  pub fn process_wait_cleared(&mut self, _: WaitCleared) -> Option<Q> {
    self.wait_until.is_some_and(|i| Instant::now() > i).then(|| self.query.clone())
  }
}

// Send enumerated requests

/// Query sender requests.
#[derive(Clone, Debug)]
pub enum QuerySenderRequest {
  UpdateQuery(QueryMessage),
}

impl<Q: Query + 'static> QuerySender<Q> {
  /// Send a [request](QuerySenderRequest), possibly returning a future producing a [response](QuerySenderResponse)
  /// that must be [processed](Self::process).
  pub fn send(&mut self, request: QuerySenderRequest) -> Option<impl Future<Output=QuerySenderResponse> + MaybeSend> {
    use QuerySenderRequest::*;
    match request {
      UpdateQuery(message) => self.update_query(message).opt_map_into().opt_boxed_maybe_send(),
    }
  }
}

// Process enumerated responses.

/// Query sender responses.
#[derive(Debug)]
pub enum QuerySenderResponse {
  WaitCleared(WaitCleared),
}
impl From<WaitCleared> for QuerySenderResponse {
  #[inline]
  fn from(e: WaitCleared) -> Self { Self::WaitCleared(e) }
}

impl<Q: Query + Clone> QuerySender<Q> {
  /// Process a [response](QuerySenderResponse), returning `Some(query)` if the query should be sent, `None` otherwise.
  pub fn process(&mut self, response: QuerySenderResponse) -> Option<Q> {
    use QuerySenderResponse::*;
    match response {
      WaitCleared(e) => self.process_wait_cleared(e),
    }
  }
}
