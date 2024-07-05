use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{CratesQuery, CratesQueryConfig, FullCrate};
use att_core::query::{Query, QueryMessage};
use att_core::service::{Action, ActionDef, Service};
use att_core::util::maybe_send::MaybeSendFuture;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Follow crates state that can be (de)serialized.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FollowCratesState {
  id_to_crate: BTreeMap<i32, FullCrate>,
}

/// Keep track of followed crates.
#[derive(Debug)]
pub struct FollowCrates {
  http_client: AttHttpClient,
  state: FollowCratesState,
  crates_being_modified: BTreeSet<i32>,
  all_crates_being_modified: bool,
  query: CratesQuery,
  crates_query_config: CratesQueryConfig,
}

impl FollowCrates {
  #[inline]
  pub fn new(http_client: AttHttpClient, state: FollowCratesState) -> Self {
    Self {
      http_client,
      state,
      crates_being_modified: Default::default(),
      all_crates_being_modified: false,
      query: CratesQuery::from_followed(),
      crates_query_config: CratesQueryConfig {
        show_followed: false,
        ..CratesQueryConfig::default()
      },
    }
  }

  #[inline]
  pub fn from_http_client(http_client: AttHttpClient) -> Self {
    Self::new(http_client, FollowCratesState::default())
  }

  #[inline]
  pub fn state(&self) -> &FollowCratesState { &self.state }
  #[inline]
  pub fn into_state(self) -> FollowCratesState { self.state }
  #[inline]
  pub fn take_state(&mut self) -> FollowCratesState { std::mem::take(&mut self.state) }


  #[inline]
  fn queried_crates(&self) -> impl Iterator<Item=&FullCrate> {
    let name = self.query.name.as_deref().unwrap_or_default();
    let follow = self.query.followed.unwrap_or(true);
    self.state.id_to_crate.values().filter(move |c| follow && c.krate.name.contains(name))
  }

  #[inline]
  fn crates_len(&self) -> usize {
    self.queried_crates().count()
  }

  #[inline]
  fn get_crates(&self, index: usize) -> Option<&FullCrate> {
    self.queried_crates().nth(index)
  }

  #[inline]
  fn iter_crates(&self) -> impl Iterator<Item=&FullCrate> {
    self.queried_crates()
  }


  #[inline]
  pub fn is_crate_being_modified(&self, crate_id: i32) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(&crate_id)
  }

  #[inline]
  pub fn are_all_crates_being_modified(&self) -> bool {
    self.all_crates_being_modified
  }
}


/// Update one crate response.
#[derive(Debug)]
pub struct UpdateOne {
  crate_id: i32,
  result: Result<FullCrate, AttHttpClientError>,
}

impl FollowCrates {
  pub fn refresh(&mut self, crate_id: i32) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.refresh_crate(crate_id);
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }

  pub fn process_update_one(&mut self, response: UpdateOne) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    let full_crate = response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id, "update crate");
    self.state.id_to_crate.insert(crate_id, full_crate);

    Ok(())
  }
}


/// Update or set all crates response.
#[derive(Debug)]
pub struct UpdateAll<const SET: bool> {
  result: Result<Vec<FullCrate>, AttHttpClientError>,
}

impl FollowCrates {
  pub fn get_followed(&mut self) -> impl Future<Output=UpdateAll<true>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.search_crates(CratesQuery::from_followed());
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn refresh_followed(&mut self) -> impl Future<Output=UpdateAll<false>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.refresh_followed();
    async move {
      UpdateAll { result: future.await }
    }
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
}

/// Follow crate response.
#[derive(Debug)]
pub struct Follow {
  full_crate: FullCrate,
  result: Result<(), AttHttpClientError>,
}

impl FollowCrates {
  pub fn follow(&mut self, full_crate: FullCrate) -> impl Future<Output=Follow> {
    let crate_id = full_crate.krate.id;
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.follow_crate(crate_id);
    async move {
      Follow { full_crate, result: future.await }
    }
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
}

/// Unfollow crate response.
#[derive(Debug)]
pub struct Unfollow {
  crate_id: i32,
  result: Result<(), AttHttpClientError>,
}

impl FollowCrates {
  pub fn unfollow(&mut self, crate_id: i32) -> impl Future<Output=Unfollow> {
    self.crates_being_modified.insert(crate_id);
    let future = self.http_client.unfollow_crate(crate_id);
    async move {
      Unfollow { crate_id, result: future.await }
    }
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
}


// Service implementation

/// Follow crate requests in message form.
#[derive(Clone, Debug)]
pub enum FollowCrateRequest {
  GetFollowed,
  Follow(FullCrate),
  Unfollow(i32),
  Refresh(i32),
  RefreshFollowed,
}

/// Follow crate responses in message form.
#[derive(Debug)]
pub enum FollowCratesResponse {
  UpdateOne(UpdateOne),
  UpdateAll(UpdateAll<false>),
  SetAll(UpdateAll<true>),
  Follow(Follow),
  Unfollow(Unfollow),
}
impl From<UpdateOne> for FollowCratesResponse {
  #[inline]
  fn from(r: UpdateOne) -> Self { Self::UpdateOne(r) }
}
impl From<UpdateAll<false>> for FollowCratesResponse {
  #[inline]
  fn from(r: UpdateAll<false>) -> Self { Self::UpdateAll(r) }
}
impl From<UpdateAll<true>> for FollowCratesResponse {
  #[inline]
  fn from(r: UpdateAll<true>) -> Self { Self::SetAll(r) }
}
impl From<Follow> for FollowCratesResponse {
  #[inline]
  fn from(r: Follow) -> Self { Self::Follow(r) }
}
impl From<Unfollow> for FollowCratesResponse {
  #[inline]
  fn from(r: Unfollow) -> Self { Self::Unfollow(r) }
}

impl Service for FollowCrates {
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
    self.crates_len()
  }

  #[inline]
  fn get_data(&self, index: usize) -> Option<&Self::Data> {
    self.get_crates(index)
  }

  #[inline]
  fn iter_data(&self) -> impl Iterator<Item=&Self::Data> {
    self.iter_crates()
  }

  type Query = CratesQuery;

  #[inline]
  fn query_config(&self) -> &<Self::Query as Query>::Config {
    &self.crates_query_config
  }

  #[inline]
  fn query(&self) -> &Self::Query {
    &self.query
  }

  #[inline]
  fn query_mut(&mut self) -> &mut Self::Query {
    &mut self.query
  }

  #[inline]
  fn update_query(&mut self, message: QueryMessage) {
    message.update_query(&mut self.query, &self.crates_query_config);
  }


  #[inline]
  fn data_action_definitions(&self) -> &[ActionDef] {
    const ICON_FONT: &'static str = "bootstrap-icons";
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_icon_font("\u{F116}", ICON_FONT),
      ActionDef::from_icon_font("\u{F5DE}", ICON_FONT).with_danger_style(),
    ];
    ACTION_DEFS
  }

  #[inline]
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


  type Request = FollowCrateRequest;

  type Response = FollowCratesResponse;

  #[inline]
  fn send(&mut self, request: Self::Request) -> impl MaybeSendFuture<'static, Output=Self::Response> + 'static {
    use FollowCrateRequest::*;
    use FollowCratesResponse::*;
    match request {
      GetFollowed => self.get_followed().map(SetAll).boxed_maybe_send(),
      FollowCrateRequest::Follow(krate) => self.follow(krate).map(FollowCratesResponse::Follow).boxed_maybe_send(),
      FollowCrateRequest::Unfollow(crate_id) => self.unfollow(crate_id).map(FollowCratesResponse::Unfollow).boxed_maybe_send(),
      Refresh(crate_id) => self.refresh(crate_id).map(UpdateOne).boxed_maybe_send(),
      RefreshFollowed => self.refresh_followed().map(UpdateAll).boxed_maybe_send(),
    }
  }

  #[inline]
  fn process(&mut self, response: Self::Response) {
    use FollowCratesResponse::*;
    match response {
      UpdateOne(r) => { let _ = self.process_update_one(r); }
      UpdateAll(r) => { let _ = self.process_update_all(r); }
      SetAll(r) => { let _ = self.process_update_all(r); }
      Follow(r) => { let _ = self.process_follow(r); }
      Unfollow(r) => { let _ = self.process_unfollow(r); }
    }
  }
}

// Service actions

enum ServiceActionKind {
  RefreshFollowed,
}

struct ServiceAction {
  kind: ServiceActionKind,
  disabled: bool,
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

// Data actions

enum DataActionKind {
  Refresh,
  Unfollow,
}

struct DataAction {
  kind: DataActionKind,
  disabled: bool,
  crate_id: i32,
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
