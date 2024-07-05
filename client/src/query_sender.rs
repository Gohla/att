use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use futures::FutureExt;

use att_core::query::{Query, QueryMessage};
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture};
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
  pub fn new(query: Q, query_config: Q::Config, wait_duration:Duration, send_query_if_empty: bool) -> Self {
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

/// Query sender requests.
#[derive(Clone, Debug)]
pub enum QuerySenderRequest {
  UpdateQuery(QueryMessage),
}

impl<Q: Query + Clone + 'static> QuerySender<Q> {
  /// Send a [request](QuerySenderRequest), possibly returning a future producing a [response](QuerySenderResponse)
  /// that must be [processed](Self::process).
  pub fn send<R>(&mut self, request: QuerySenderRequest) -> Option<impl Future<Output=QuerySenderResponse<R>> + MaybeSend> {
    use QuerySenderRequest::*;
    use QuerySenderResponse::*;
    match request {
      UpdateQuery(message) => self.update_query(message).map(|f| f.map(|_| WaitCleared()).boxed_maybe_send()),
    }
  }

  /// Update the query from `message`, returning a future producing a [response](WaitCleared) that must be
  /// [processed](Self::process_wait_cleared).
  ///
  /// If `send_query_if_empty` is `false` and the query is empty: returns `None` and any ongoing query will not be sent
  /// when the wait is cleared.
  fn update_query(&mut self, message: QueryMessage) -> Option<impl Future<Output=()>> {
    message.update_query(&mut self.query, &self.query_config);
    if !self.send_query_if_empty && self.query.is_empty(&self.query_config) {
      self.wait_until = None;
      None
    } else {
      let wait_until = Instant::now() + self.wait_duration;
      self.wait_until = Some(wait_until);
      let future = sleep(self.wait_duration);
      Some(future)
    }
  }
}

/// Query sender responses.
#[derive(Clone, Debug)]
pub enum QuerySenderResponse<R> {
  WaitCleared(),
  QueryResult(R),
}

/// Result of processing a response.
pub enum ProcessResult<Q, R> {
  SendQuery(Q),
  QueryResult(R),
}

impl<Q: Query + Clone + 'static> QuerySender<Q> {
  /// Process a [response](QuerySenderResponse), possibly returning a request that must be [sent](Self::send).
  pub fn process<R>(&mut self, response: QuerySenderResponse<R>) -> Option<ProcessResult<Q, R>> {
    use QuerySenderResponse::*;
    match response {
      WaitCleared() => self.should_send_query().then(|| ProcessResult::SendQuery(self.query.clone())),
      QueryResult(r) => Some(ProcessResult::QueryResult(r)),
    }
  }

  /// Checks whether the query should be sent now.
  #[inline]
  fn should_send_query(&self) -> bool {
    self.wait_until.is_some_and(|i| Instant::now() > i)
  }
}
