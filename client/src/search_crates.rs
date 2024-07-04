use std::future::Future;
use std::time::Duration;

use futures::FutureExt;
use tracing::{debug, error};

use att_core::crates::{CratesQuery, FullCrate};
use att_core::query::Query;
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture};
use att_core::util::time::{Instant, sleep};

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Search for crates.
#[derive(Debug)]
pub struct SearchCrates {
  http_client: AttHttpClient,
  search_query: CratesQuery,
  wait_until: Option<Instant>,
  found_crates: Vec<FullCrate>,
}
impl SearchCrates {
  pub fn new(http_client: AttHttpClient) -> Self {
    Self {
      http_client,
      search_query: CratesQuery::default(),
      wait_until: None,
      found_crates: Vec::new()
    }
  }

  /// Returns the [search query](CratesQuery).
  #[inline]
  pub fn search_query(&self) -> &CratesQuery { &self.search_query }
  /// Returns the search term of the search query.
  #[inline]
  pub fn search_term(&self) -> &str { &self.search_query.name.as_deref().unwrap_or_default() }
  /// Returns the found crates returned by the latest search.
  #[inline]
  pub fn found_crates(&self) -> &Vec<FullCrate> { &self.found_crates }

  /// Set the [search query](CratesQuery), possibly returning a future producing a [response](WaitCleared) that
  /// must be [processed](Self::process_wait_cleared).
  pub fn set_search_query(&mut self, search_query: CratesQuery) -> Option<impl Future<Output=WaitCleared>> {
    self.search_query = search_query;
    if self.search_query.is_empty() {
      self.wait_until = None;
      self.found_crates.clear();
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
  /// Process a [wait cleared response](WaitCleared), possibly returning a future producing a [response](FoundCrates)
  /// that must be [processed](Self::process_found_crates).
  pub fn process_wait_cleared(&self, _response: WaitCleared) -> Option<impl Future<Output=FoundCrates>> {
    self.should_search().then(|| self.search())
  }
  /// Checks whether a search request should be sent now.
  #[inline]
  pub fn should_search(&self) -> bool {
    self.wait_until.is_some_and(|i| Instant::now() > i)
  }

  /// Sends a search request with the current search query, returning a future producing a [response](FoundCrates) that
  /// must be [processed](Self::process_found_crates).
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
    debug!("found {} crates", crates.len());
    self.found_crates = crates;

    Ok(())
  }

  /// Clears the search query and found crates, and cancels ongoing searches.
  pub fn clear(&mut self) {
    self.search_query = CratesQuery::default();
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
  result: Result<Vec<FullCrate>, AttHttpClientError>
}


/// Search crate requests in message form.
#[derive(Debug)]
pub enum SearchCratesRequest {
  SetSearchQuery(CratesQuery),
  Search,
}
impl SearchCrates {
  /// Create a request for setting the [`search_term` of the search query](CratesQuery::search_term).
  pub fn request_set_search_term(&self, search_term: String) -> SearchCratesRequest {
    let mut search_query = self.search_query.clone();
    search_query.name = Some(search_term);
    SearchCratesRequest::SetSearchQuery(search_query)
  }

  /// Send a [request](SearchCratesRequest), possibly returning a future producing a [response](SearchCratesResponse)
  /// that must be [processed](Self::process).
  pub fn send(&mut self, request: SearchCratesRequest) -> Option<impl Future<Output=SearchCratesResponse> + MaybeSend> {
    use SearchCratesRequest::*;
    use SearchCratesResponse::*;
    match request {
      SetSearchQuery(search_query) => self.set_search_query(search_query).map(|f| f.map(WaitCleared).boxed_maybe_send()),
      Search => Some(self.search().map(FoundCrates).boxed_maybe_send()),
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
  /// Process a [response](SearchCratesResponse), possibly returning a request that must be [sent](Self::send).
  pub fn process(&mut self, response: SearchCratesResponse) -> Option<SearchCratesRequest> {
    use SearchCratesResponse::*;
    match response {
      WaitCleared(_) => return self.should_search().then_some(SearchCratesRequest::Search),
      FoundCrates(r) => { let _ = self.process_found_crates(r); },
    }
    None
  }
}
