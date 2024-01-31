use std::future::Future;
use std::time::Duration;

use futures::FutureExt;
use tracing::{debug, error};

use att_core::crates::{Crate, CrateSearchQuery};
use att_core::util::maybe_send::MaybeSendFuture;
use att_core::util::time::{Instant, sleep};

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Search for crates.
#[derive(Debug)]
pub struct SearchCrates {
  http_client: AttHttpClient,
  search_query: CrateSearchQuery,
  wait_until: Option<Instant>,
  found_crates: Vec<Crate>,
}
impl SearchCrates {
  pub fn new(http_client: AttHttpClient) -> Self {
    Self {
      http_client,
      search_query: CrateSearchQuery::default(),
      wait_until: None,
      found_crates: Vec::new()
    }
  }

  /// Returns the [search query](CrateSearchQuery).
  #[inline]
  pub fn search_query(&self) -> &CrateSearchQuery { &self.search_query }
  /// Returns the search term of the search query.
  #[inline]
  pub fn search_term(&self) -> &str { &self.search_query.search_term() }
  /// Returns the found crates returned by the latest search.
  #[inline]
  pub fn found_crates(&self) -> &Vec<Crate> { &self.found_crates }

  /// Set the [crate search query](CrateSearchQuery), returning a future producing a [response](WaitCleared) that must
  /// be [processed](Self::process_wait_cleared).
  pub fn set_search_query(&mut self, search_query: CrateSearchQuery) -> impl Future<Output=WaitCleared> {
    self.search_query = search_query;
    let wait_duration = Duration::from_millis(300);
    let wait_until = Instant::now() + wait_duration;
    self.wait_until = Some(wait_until);
    let future = sleep(wait_duration);
    async move {
      future.await;
      WaitCleared
    }
  }
  /// Process a [wait cleared response](WaitCleared), possibly returning a future producing a [response](FoundCrates)
  /// that must be [processed](Self::process_found_crates).
  pub fn process_wait_cleared(&self, _response: WaitCleared) -> Option<impl Future<Output=FoundCrates>> {
    if let Some(wait_until) = self.wait_until {
      if Instant::now() > wait_until {
        return Some(self.search());
      }
    }
    None
  }

  /// Perform a search with the current search query, returning a future producing a [response](FoundCrates) that must
  /// be [processed](Self::process_found_crates).
  pub fn search(&self) -> impl Future<Output=FoundCrates> {
    let future = self.http_client.search_crates(self.search_query.clone());
    async move {
      FoundCrates { result: future.await }
    }
  }
  /// Processes a [found crates response](FoundCrates), updating the [found crates](Self::found_crates).
  pub fn process_found_crates(&mut self, response: FoundCrates) -> Result<(), AttHttpClientError> {
    let crates = response.result
      .inspect_err(|cause| error!(%cause, "failed to search for crates: {cause:?}"))?;
    debug!(?crates, "found crates");
    self.found_crates = crates;

    Ok(())
  }

  /// Clears the search query and found crates, and cancels ongoing searches.
  pub fn clear(&mut self) {
    self.search_query = CrateSearchQuery::default();
    self.wait_until = None;
    self.found_crates.clear();
  }
}

/// Wait time cleared response.
#[derive(Debug)]
pub struct WaitCleared;

/// Found crates by search response.
#[derive(Debug)]
pub struct FoundCrates {
  result: Result<Vec<Crate>, AttHttpClientError>
}


/// Search crate requests in message form.
#[derive(Debug)]
pub enum SearchCratesRequest {
  SetSearchQuery(CrateSearchQuery),
  Search,
}
impl SearchCrates {
  /// Create a request for setting the [`search_term` of the search query](CrateSearchQuery::search_term).
  pub fn request_set_search_term(&self, search_term: String) -> SearchCratesRequest {
    let mut search_query = self.search_query.clone();
    search_query.search_term = Some(search_term);
    SearchCratesRequest::SetSearchQuery(search_query)
  }

  /// Send a [request](SearchCratesRequest), returning a future producing a [response](SearchCratesResponse) that must
  /// be [processed](Self::process).
  pub fn send(&mut self, request: SearchCratesRequest) -> impl MaybeSendFuture<'static, Output=SearchCratesResponse> {
    use SearchCratesRequest::*;
    use SearchCratesResponse::*;
    match request {
      SetSearchQuery(search_query) => self.set_search_query(search_query).map(WaitCleared).boxed_maybe_send(),
      Search => self.search().map(FoundCrates).boxed_maybe_send(),
    }
  }
}

/// Search crate responses in message form.
#[derive(Debug)]
pub enum SearchCratesResponse {
  WaitCleared(WaitCleared),
  FoundCrates(FoundCrates),
}
impl From<WaitCleared> for SearchCratesResponse {
  #[inline]
  fn from(r: WaitCleared) -> Self { Self::WaitCleared(r) }
}
impl From<FoundCrates> for SearchCratesResponse {
  #[inline]
  fn from(r: FoundCrates) -> Self { Self::FoundCrates(r) }
}
impl SearchCrates {
  /// Process a [response](SearchCratesResponse), possibly returning a future producing a
  /// [response](SearchCratesResponse) that must be [processed](Self::process).
  pub fn process(&mut self, response: SearchCratesResponse) -> Option<impl MaybeSendFuture<'static, Output=SearchCratesResponse>> {
    use SearchCratesResponse::*;
    match response {
      WaitCleared(r) => return self.process_wait_cleared(r).map(|f| f.map(FoundCrates).boxed_maybe_send()),
      FoundCrates(r) => { let _ = self.process_found_crates(r); },
    }
    None
  }
}
