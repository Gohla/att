use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;

use futures::FutureExt;

use att_core::query::{Query, QueryMessage};
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture};
use att_core::util::time::{Instant, sleep};

#[derive(Debug)]
pub struct QuerySender<Q: Query, R> {
  query: Q,
  query_config: Q::Config,
  wait_until: Option<Instant>,
  send_query_if_empty: bool,
  _result_phantom: PhantomData<R>,
}
impl<Q, R> QuerySender<Q, R> where
  Q: Query + Clone + 'static,
  R: 'static,
{
  pub fn new(
    query: Q,
    query_config: Q::Config,
    send_query_if_empty: bool,
  ) -> Self {
    Self {
      query,
      query_config,
      wait_until: None,
      send_query_if_empty,
      _result_phantom: PhantomData,
    }
  }


  /// Returns the query.
  #[inline]
  pub fn query(&self) -> &Q { &self.query }

  /// Returns the query.
  #[inline]
  pub fn query_config(&self) -> &Q::Config { &self.query_config }


  /// Send a [request](QuerySenderRequest), possibly returning a future producing a [response](QuerySenderResponse)
  /// that must be [processed](Self::process).
  pub fn send(&mut self, request: QuerySenderRequest) -> Option<impl Future<Output=QuerySenderResponse<R>> + MaybeSend> {
    use QuerySenderRequest::*;
    use QuerySenderResponse::*;
    match request {
      UpdateQuery(message) => self.update_query(message).map(|f| f.map(|_|WaitCleared()).boxed_maybe_send()),
    }
  }

  /// Process a [response](QuerySenderResponse), possibly returning a request that must be [sent](Self::send).
  pub fn process(&mut self, response: QuerySenderResponse<R>) -> Option<ProcessOutput<Q, R>> {
    use QuerySenderResponse::*;
    match response {
      WaitCleared() => self.should_send_query().then(||ProcessOutput::SendQuery(self.query.clone())),
      QueryResult(r) => Some(ProcessOutput::QueryResult(r)),
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
      let wait_duration = Duration::from_millis(300);
      let wait_until = Instant::now() + wait_duration;
      self.wait_until = Some(wait_until);
      let future = sleep(wait_duration);
      Some(future)
    }
  }

  /// Checks whether the query should be sent now.
  #[inline]
  fn should_send_query(&self) -> bool {
    self.wait_until.is_some_and(|i| Instant::now() > i)
  }
}

pub enum ProcessOutput<Q, R> {
  SendQuery(Q),
  QueryResult(R),
}

// /// Wait time cleared response.
// #[derive(Clone, Debug)]
// pub struct WaitCleared;
//
// /// Data from query response.
// #[derive(Clone, Debug)]
// pub struct QueryResult<R> {
//   result: R,
// }

/// Search crate requests in message form.
#[derive(Clone, Debug)]
pub enum QuerySenderRequest {
  UpdateQuery(QueryMessage),
}

/// Search crate responses in message form.
#[derive(Clone, Debug)]
pub enum QuerySenderResponse<R> {
  WaitCleared(),
  QueryResult(R),
}
// impl<R> From<WaitCleared> for QuerySenderResponse<R> {
//   #[inline]
//   fn from(r: WaitCleared) -> Self { Self::WaitCleared(r) }
// }
// impl<R> From<QueryResult<R>> for QuerySenderResponse<R> {
//   #[inline]
//   fn from(r: QueryResult<R>) -> Self { Self::QueryResult(r) }
// }
