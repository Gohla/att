use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use futures::FutureExt;

use att_core::query::{Query, QueryMessage};
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture};
use att_core::util::time::{Instant, sleep};

#[derive(Debug)]
pub struct SearchQuery<Q, F> {
  query: Q,
  wait_until: Option<Instant>,
  create_future: F,
}
impl<Q, F, R, Fut> SearchQuery<Q, F> where
  Q: Query + Clone + 'static,
  F: Fn(Q) -> Fut + 'static,
  R: 'static,
  Fut: Future<Output=R> + Send + 'static,
{
  pub fn new(query: Q, create_future: F) -> Self {
    Self {
      query,
      wait_until: None,
      create_future,
    }
  }


  /// Returns the query.
  #[inline]
  pub fn query(&self) -> &Q { &self.query }

  /// Returns the mutable query.
  #[inline]
  pub fn query_mut(&mut self) -> &mut Q { &mut self.query }


  /// Update the query from `message`, returning `Some(future)` producing a [response](WaitCleared) that
  /// must be [processed](Self::process_wait_cleared) if the query is not empty. Returns `None` if the query is empty.
  pub fn update_query(&mut self, message: QueryMessage) -> Option<impl Future<Output=WaitCleared>> {
    message.update_query(&mut self.query);
    if self.query.is_empty() {
      self.wait_until = None;
      None
    } else {
      let wait_duration = Duration::from_millis(300);
      let wait_until = Instant::now() + wait_duration;
      self.wait_until = Some(wait_until);
      let future = sleep(wait_duration);
      Some(async move {
        future.await;
        WaitCleared
      })
    }
  }

  /// Process a [wait cleared response](WaitCleared), possibly returning a future producing a [response](QueryResult)
  /// that must be [processed](Self::process_result).
  pub fn process_wait_cleared(&self, _response: WaitCleared) -> Option<impl Future<Output=QueryResult<R>>> {
    self.should_send_query().then(|| self.send_current_query())
  }

  /// Checks whether the query should be sent now.
  #[inline]
  pub fn should_send_query(&self) -> bool {
    self.wait_until.is_some_and(|i| Instant::now() > i)
  }


  /// Sends the current query, returning a future producing a [response](QueryResult) that must be
  /// [processed](Self::process_result).
  pub fn send_current_query(&self) -> impl Future<Output=QueryResult<R>> {
    let future = (self.create_future)(self.query.clone());
    async move {
      QueryResult { result: future.await }
    }
  }

  /// Processes a [query result response](QueryResult), returning the result.
  pub fn process_result(&mut self, response: QueryResult<R>) -> R {
    response.result
  }


  /// Send a [request](QueryRequest), possibly returning a future producing a [response](QueryResponse)
  /// that must be [processed](Self::process).
  pub fn send(&mut self, request: QueryRequest) -> Option<impl Future<Output=QueryResponse<R>> + MaybeSend> {
    use QueryRequest::*;
    use QueryResponse::*;
    match request {
      UpdateQuery(message) => self.update_query(message).map(|f| f.map(WaitCleared).boxed_maybe_send()),
      SendCurrentQuery => Some(self.send_current_query().map(QueryResult).boxed_maybe_send()),
    }
  }

  /// Process a [response](QueryResponse), possibly returning a request that must be [sent](Self::send_current_query).
  pub fn process(&mut self, response: QueryResponse<R>) -> Option<QueryRequest> {
    use QueryResponse::*;
    match response {
      WaitCleared(_) => return self.should_send_query().then_some(QueryRequest::SendCurrentQuery),
      QueryResult(r) => { let _ = self.process_result(r); },
    }
    None
  }
}

/// Wait time cleared response.
#[derive(Debug)]
pub struct WaitCleared;

/// Data from query response.
#[derive(Debug)]
pub struct QueryResult<R> {
  result: R,
}

/// Search crate requests in message form.
#[derive(Debug)]
pub enum QueryRequest {
  UpdateQuery(QueryMessage),
  SendCurrentQuery,
}

/// Search crate responses in message form.
#[derive(Debug)]
pub enum QueryResponse<R> {
  WaitCleared(WaitCleared),
  QueryResult(QueryResult<R>),
}
impl<R> From<WaitCleared> for QueryResponse<R> {
  #[inline]
  fn from(r: WaitCleared) -> Self { Self::WaitCleared(r) }
}
impl<R> From<QueryResult<R>> for QueryResponse<R> {
  #[inline]
  fn from(r: QueryResult<R>) -> Self { Self::QueryResult(r) }
}
