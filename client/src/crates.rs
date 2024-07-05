use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::time::Duration;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{CratesQuery, CratesQueryConfig, FullCrate};
use att_core::query::{Query, QueryMessage};
use att_core::service::{Action, ActionDef, Service};
use att_core::util::future::OptFutureExt;
use att_core::util::maybe_send::{MaybeSend, MaybeSendFuture, MaybeSendOptFuture};

use crate::http_client::{AttHttpClient, AttHttpClientError};
use crate::query_sender::{QuerySender, QuerySenderRequest, QuerySenderResponse};

/// Crates state that can be (de)serialized.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CratesState {
  id_to_crate: BTreeMap<i32, FullCrate>,
}

/// Keep track of crates.
#[derive(Debug)]
pub struct Crates {
  http_client: AttHttpClient,
  state: CratesState,
  crates_being_modified: BTreeSet<i32>,
  all_crates_being_modified: bool,
  query_sender: QuerySender<CratesQuery>,
}

impl Crates {
  #[inline]
  pub fn new(http_client: AttHttpClient, state: CratesState) -> Self {
    Self {
      http_client,
      state,
      crates_being_modified: Default::default(),
      all_crates_being_modified: false,
      query_sender: QuerySender::new(
        CratesQuery::from_followed(),
        CratesQueryConfig {
          show_followed: false,
          ..CratesQueryConfig::default()
        },
        Duration::from_millis(300),
        true,
      ),
    }
  }

  #[inline]
  pub fn from_http_client(http_client: AttHttpClient) -> Self {
    Self::new(http_client, CratesState::default())
  }

  #[inline]
  pub fn state(&self) -> &CratesState { &self.state }
}

// Send specific requests

impl Crates {
  pub fn send_initial_query(&mut self) -> impl Future<Output=UpdateAll<true>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.search_crates(self.query_sender.query().clone());
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn send_refresh(&mut self, crate_id: i32) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.refresh_crate(crate_id);
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }

  pub fn send_refresh_followed(&mut self) -> impl Future<Output=UpdateAll<false>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.refresh_followed();
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn send_follow(&mut self, full_crate: FullCrate) -> impl Future<Output=Follow> {
    let crate_id = full_crate.krate.id;
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.follow_crate(crate_id);
    async move {
      Follow { full_crate, result: future.await }
    }
  }

  pub fn send_unfollow(&mut self, crate_id: i32) -> impl Future<Output=Unfollow> {
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.unfollow_crate(crate_id);
    async move {
      Unfollow { crate_id, result: future.await }
    }
  }

  pub fn send_query(
    &mut self,
    request: QuerySenderRequest
  ) -> Option<impl Future<Output=QuerySenderResponse>> {
    self.query_sender.send(request)
  }
}

// Process specific responses

/// Update one crate response.
#[derive(Debug)]
pub struct UpdateOne {
  crate_id: i32,
  result: Result<FullCrate, AttHttpClientError>,
}

pub type FullCratesResult = Result<Vec<FullCrate>, AttHttpClientError>;

/// Update or set all crates response.
#[derive(Debug)]
pub struct UpdateAll<const SET: bool> {
  result: FullCratesResult,
}

/// Follow crate response.
#[derive(Debug)]
pub struct Follow {
  full_crate: FullCrate,
  result: Result<(), AttHttpClientError>,
}

/// Unfollow crate response.
#[derive(Debug)]
pub struct Unfollow {
  crate_id: i32,
  result: Result<(), AttHttpClientError>,
}

impl Crates {
  pub fn process_update_one(&mut self, response: UpdateOne) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    let full_crate = response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id, "update crate");
    self.state.id_to_crate.insert(crate_id, full_crate);

    Ok(())
  }

  pub fn process_update_all<const SET: bool>(&mut self, response: UpdateAll<SET>) -> Result<(), AttHttpClientError> {
    self.all_crates_being_modified = false;

    let full_crates = response.result
      .inspect_err(|cause| error!(%cause, "failed to update crates: {cause:?}"))?;
    if SET {
      self.state.id_to_crate.clear();
    }
    for full_crate in full_crates {
      debug!(crate_id = full_crate.krate.id, "update crate");
      self.state.id_to_crate.insert(full_crate.krate.id, full_crate);
    }

    Ok(())
  }

  pub fn process_follow(&mut self, response: Follow) -> Result<(), AttHttpClientError> {
    let crate_id = response.full_crate.krate.id;
    self.crates_being_modified.remove(&crate_id);

    response.result
      .inspect_err(|cause| error!(crate = ?response.full_crate, %cause, "failed to follow crate: {cause:?}"))?;
    debug!(crate = ?response.full_crate, "follow crate");
    self.state.id_to_crate.insert(crate_id, response.full_crate);

    Ok(())
  }

  pub fn process_unfollow(&mut self, response: Unfollow) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to unfollow crate: {cause:?}"))?;
    debug!(crate_id, "unfollow crate");
    self.state.id_to_crate.remove(&crate_id);

    Ok(())
  }

  pub fn process_query(&mut self, response: QuerySenderResponse) -> Option<impl Future<Output=UpdateAll<true>>> {
    match self.query_sender.process(response) {
      Some(query) => {
        let future = self.http_client
          .search_crates(query)
          .map(|result| UpdateAll { result });
        return Some(future);
      },
      None => None,
    }
  }
}

// Send enumerated requests

/// Crate requests.
#[derive(Clone, Debug)]
pub enum FollowCrateRequest {
  InitialQuery,
  Follow(FullCrate),
  Unfollow(i32),
  Refresh(i32),
  RefreshFollowed,
  Query(QuerySenderRequest),
}

impl Crates {
  pub fn send(
    &mut self,
    request: FollowCrateRequest
  ) -> Option<impl Future<Output=FollowCratesResponse> + MaybeSend + 'static> {
    use FollowCrateRequest::*;
    let future = match request {
      InitialQuery => self.send_initial_query().map_into().boxed_maybe_send(),
      Follow(krate) => self.send_follow(krate).map_into().boxed_maybe_send(),
      Unfollow(crate_id) => self.send_unfollow(crate_id).map_into().boxed_maybe_send(),
      Refresh(crate_id) => self.send_refresh(crate_id).map_into().boxed_maybe_send(),
      RefreshFollowed => self.send_refresh_followed().map_into().boxed_maybe_send(),
      Query(r) => return self.send_query(r).opt_map_into().opt_boxed_maybe_send(),
    };
    Some(future)
  }
}

// Process enumerated responses

/// Crate responses.
#[derive(Debug)]
pub enum FollowCratesResponse {
  UpdateOne(UpdateOne),
  UpdateAll(UpdateAll<false>),
  SetAll(UpdateAll<true>),
  Follow(Follow),
  Unfollow(Unfollow),
  Query(QuerySenderResponse),
}
impl From<UpdateOne> for FollowCratesResponse {
  #[inline]
  fn from(e: UpdateOne) -> Self { Self::UpdateOne(e) }
}
impl From<UpdateAll<false>> for FollowCratesResponse {
  #[inline]
  fn from(e: UpdateAll<false>) -> Self { Self::UpdateAll(e) }
}
impl From<UpdateAll<true>> for FollowCratesResponse {
  #[inline]
  fn from(e: UpdateAll<true>) -> Self { Self::SetAll(e) }
}
impl From<Follow> for FollowCratesResponse {
  #[inline]
  fn from(e: Follow) -> Self { Self::Follow(e) }
}
impl From<Unfollow> for FollowCratesResponse {
  #[inline]
  fn from(e: Unfollow) -> Self { Self::Unfollow(e) }
}
impl From<QuerySenderResponse> for FollowCratesResponse {
  #[inline]
  fn from(e: QuerySenderResponse) -> Self { Self::Query(e) }
}

impl Crates {
  pub fn process(
    &mut self,
    response: FollowCratesResponse
  ) -> Option<impl Future<Output=FollowCratesResponse> + MaybeSend + 'static> {
    use FollowCratesResponse::*;
    match response {
      UpdateOne(e) => { let _ = self.process_update_one(e); }
      UpdateAll(e) => { let _ = self.process_update_all(e); }
      SetAll(e) => { let _ = self.process_update_all(e); }
      Follow(e) => { let _ = self.process_follow(e); }
      Unfollow(e) => { let _ = self.process_unfollow(e); }
      Query(e) => return self.process_query(e).opt_map_into(),
    }
    None
  }
}

// Service implementation

impl Service for Crates {
  type Request = FollowCrateRequest;
  type Response = FollowCratesResponse;

  #[inline]
  fn send(&mut self, request: Self::Request) -> Option<impl Future<Output=Self::Response> + MaybeSend + 'static> {
    Crates::send(self, request)
  }
  #[inline]
  fn process(&mut self, response: Self::Response) -> Option<impl Future<Output=Self::Response> + MaybeSend + 'static> {
    Crates::process(self, response)
  }


  fn action_definitions(&self) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_text("Refresh Followed"),
    ];
    ACTION_DEFS
  }
  fn actions(&self) -> impl IntoIterator<Item=impl Action<Request=Self::Request>> {
    let disabled = self.are_all_crates_being_modified();
    [
      ServiceAction { kind: ServiceActionKind::RefreshFollowed, disabled },
    ]
  }


  type Data = FullCrate;

  #[inline]
  fn data_len(&self) -> usize {
    self.state.id_to_crate.len()
  }
  #[inline]
  fn get_data(&self, index: usize) -> Option<&Self::Data> {
    // OPTO: instead of going through iterator with `nth`, can we directly go to the index efficiently?
    self.state.id_to_crate.values().nth(index)
  }
  #[inline]
  fn iter_data(&self) -> impl Iterator<Item=&Self::Data> {
    self.state.id_to_crate.values()
  }


  type Query = CratesQuery;

  #[inline]
  fn query_config(&self) -> &<Self::Query as Query>::Config {
    &self.query_sender.query_config()
  }
  #[inline]
  fn query(&self) -> &Self::Query {
    &self.query_sender.query()
  }
  #[inline]
  fn request_query_update(&self, message: QueryMessage) -> Self::Request {
    FollowCrateRequest::Query(QuerySenderRequest::UpdateQuery(message))
  }


  fn data_action_definitions(&self) -> &[ActionDef] {
    const ICON_FONT: &'static str = "bootstrap-icons";
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_icon_font("\u{F116}", ICON_FONT),
      ActionDef::from_icon_font("\u{F5DE}", ICON_FONT).with_danger_style(),
    ];
    ACTION_DEFS
  }
  fn data_action<'i>(&self, index: usize, data: &'i Self::Data) -> Option<impl Action<Request=Self::Request> + 'i> {
    let crate_id = data.krate.id;
    let disabled = self.is_crate_being_modified(crate_id);
    let action = match index {
      0 => DataAction { kind: DataActionKind::Refresh, disabled, crate_id },
      1 => DataAction { kind: DataActionKind::Unfollow, disabled, crate_id },
      _ => return None,
    };
    Some(action)
  }
}

// Actions

impl Crates {
  #[inline]
  fn is_crate_being_modified(&self, crate_id: i32) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(&crate_id)
  }
  #[inline]
  fn are_all_crates_being_modified(&self) -> bool {
    self.all_crates_being_modified
  }
}

/// Crates service actions
struct ServiceAction {
  kind: ServiceActionKind,
  disabled: bool,
}

enum ServiceActionKind {
  RefreshFollowed,
}

impl Action for ServiceAction {
  type Request = FollowCrateRequest;

  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> FollowCrateRequest {
    match self.kind {
      ServiceActionKind::RefreshFollowed => FollowCrateRequest::RefreshFollowed,
    }
  }
}

/// Crates data actions.
struct DataAction {
  kind: DataActionKind,
  disabled: bool,
  crate_id: i32,
}

enum DataActionKind {
  Refresh,
  Unfollow,
}

impl Action for DataAction {
  type Request = FollowCrateRequest;

  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> FollowCrateRequest {
    match self.kind {
      DataActionKind::Refresh => FollowCrateRequest::Refresh(self.crate_id),
      DataActionKind::Unfollow => FollowCrateRequest::Unfollow(self.crate_id),
    }
  }
}
