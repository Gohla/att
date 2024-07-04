use std::fmt::{Debug, Display};
use std::future::Future;
use std::time::Duration;

use futures::FutureExt;
use tracing::{debug, error};

use att_core::query::{Query, QueryDef, QueryMessage};
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture};
use att_core::util::time::{Instant, sleep};

#[derive(Debug)]
pub struct SearchQuery<T, F> {
  query_def: QueryDef,
  create_future: F,

  query: Query,
  wait_until: Option<Instant>,
  data: Vec<T>,
}
impl<T, E, Fut, F> SearchQuery<T, F> where
  T: 'static,
  E: Display + Debug + 'static,
  Fut: Future<Output=Result<Vec<T>, E>> + Send + 'static,
  F: Fn(Query) -> Fut + 'static
{
  pub fn new(query_def: QueryDef, create_future: F) -> Self {
    let query = query_def.create_query();
    Self {
      query_def,
      create_future,

      query,
      wait_until: None,
      data: Vec::default(),
    }
  }


  /// Returns the query.
  #[inline]
  pub fn query_def(&self) -> &QueryDef { &self.query_def }

  /// Returns the query.
  #[inline]
  pub fn query(&self) -> &Query { &self.query }

  /// Returns the data.
  #[inline]
  pub fn data(&self) -> &[T] { &self.data }


  /// Update the query from `message`, possibly returning a future producing a [response](WaitCleared) that
  /// must be [processed](Self::process_wait_cleared).
  pub fn update_query(&mut self, message: QueryMessage) -> Option<impl Future<Output=WaitCleared>> {
    message.update_query(&mut self.query);
    if self.query.is_empty() {
      self.wait_until = None;
      self.data.clear();
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
  pub fn process_wait_cleared(&self, _response: WaitCleared) -> Option<impl Future<Output=QueryResult<T, E>>> {
    self.should_send_query().then(|| self.send_current_query())
  }

  /// Checks whether the query should be sent now.
  #[inline]
  pub fn should_send_query(&self) -> bool {
    self.wait_until.is_some_and(|i| Instant::now() > i)
  }


  /// Sends the current query, returning a future producing a [response](QueryResult) that must be
  /// [processed](Self::process_result).
  pub fn send_current_query(&self) -> impl Future<Output=QueryResult<T, E>> {
    let future = (self.create_future)(self.query.clone());
    async move {
      QueryResult { result: future.await }
    }
  }

  /// Processes a [query result response](QueryResult), updating the data.
  pub fn process_result(&mut self, response: QueryResult<T, E>) -> Result<(), E> {
    let data = response.result
      .inspect_err(|cause| error!(?cause, "failed to query: {cause}"))?;
    debug!("query resulted in {} data elements", data.len());
    self.data = data;

    Ok(())
  }


  /// Send a [request](QueryRequest), possibly returning a future producing a [response](QueryResponse)
  /// that must be [processed](Self::process).
  pub fn send(&mut self, request: QueryRequest) -> Option<impl Future<Output=QueryResponse<T, E>> + MaybeSend> {
    use QueryRequest::*;
    use QueryResponse::*;
    match request {
      UpdateQuery(message) => self.update_query(message).map(|f| f.map(WaitCleared).boxed_maybe_send()),
      SendCurrentQuery => Some(self.send_current_query().map(QueryResult).boxed_maybe_send()),
    }
  }

  /// Process a [response](QueryResponse), possibly returning a request that must be [sent](Self::send_current_query).
  pub fn process(&mut self, response: QueryResponse<T, E>) -> Option<QueryRequest> {
    use QueryResponse::*;
    match response {
      WaitCleared(_) => return self.should_send_query().then_some(QueryRequest::SendCurrentQuery),
      QueryResult(r) => { let _ = self.process_result(r); },
    }
    None
  }


  /// Clears the query and data, and cancels ongoing queries.
  pub fn clear(&mut self) {
    self.query = self.query_def.create_query(); // OPTO: reduce allocation by reusing query.
    self.wait_until = None;
    self.data.clear();
  }
}

/// Wait time cleared response.
#[derive(Debug)]
pub struct WaitCleared;

/// Data from query response.
#[derive(Debug)]
pub struct QueryResult<T, E> {
  result: Result<Vec<T>, E>,
}

/// Search crate requests in message form.
#[derive(Debug)]
pub enum QueryRequest {
  UpdateQuery(QueryMessage),
  SendCurrentQuery,
}

/// Search crate responses in message form.
#[derive(Debug)]
pub enum QueryResponse<T, E> {
  WaitCleared(WaitCleared),
  QueryResult(QueryResult<T, E>),
}
impl<T, E> From<WaitCleared> for QueryResponse<T, E> {
  #[inline]
  fn from(r: WaitCleared) -> Self { Self::WaitCleared(r) }
}
impl<T, E> From<QueryResult<T, E>> for QueryResponse<T, E> {
  #[inline]
  fn from(r: QueryResult<T, E>) -> Self { Self::QueryResult(r) }
}
