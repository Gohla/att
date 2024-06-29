use std::collections::BTreeSet;
use std::future::Future;

use futures::FutureExt;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{Crate, CrateSearchQuery};
use att_core::query::{FacetDef, FacetType, Query, QueryDef};
use att_core::service::{Action, ActionDef, Service};
use att_core::util::maybe_send::MaybeSendFuture;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Follow crates state that can be (de)serialized.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FollowCratesState {
  id_to_crate: IndexMap<String, Crate>,
}

/// Keep track of followed crates.
#[derive(Debug)]
pub struct FollowCrates {
  http_client: AttHttpClient,
  state: FollowCratesState,
  crates_being_modified: BTreeSet<String>,
  all_crates_being_modified: bool,
  query_def: QueryDef,
  query: Query,
}

impl FollowCrates {
  #[inline]
  pub fn new(http_client: AttHttpClient, state: FollowCratesState) -> Self {
    let query_def = QueryDef::default()
      .with_facet_def("follow", FacetDef::new("Following", FacetType::Boolean { default_value: Some(true) }))
      .with_facet_def("name", FacetDef::new("Name", FacetType::String { default_value: Some(String::new()), placeholder: Some("Crate name contains...") }));
    Self {
      http_client,
      state,
      crates_being_modified: Default::default(),
      all_crates_being_modified: false,
      query: query_def.create_query(),
      query_def,
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
  fn queried_crates(&self) -> impl Iterator<Item=&Crate> {
    let name = self.query.facet("name").unwrap().as_str().unwrap_or_default();
    let follow = self.query.facet("follow").unwrap().as_bool().unwrap_or(true);
    self.state.id_to_crate.values().filter(move |c| follow && c.id.contains(name))
  }

  #[inline]
  fn crates_len(&self) -> usize {
    self.queried_crates().count()
  }

  #[inline]
  fn get_crates(&self, index: usize) -> Option<&Crate> {
    self.queried_crates().nth(index)
  }

  #[inline]
  fn iter_crates(&self) -> impl Iterator<Item=&Crate> {
    self.queried_crates()
  }


  #[inline]
  pub fn is_crate_being_modified(&self, crate_id: &str) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(crate_id)
  }

  #[inline]
  pub fn are_all_crates_being_modified(&self) -> bool {
    self.all_crates_being_modified
  }
}


/// Update one crate response.
#[derive(Debug)]
pub struct UpdateOne {
  crate_id: String,
  result: Result<Crate, AttHttpClientError>,
}

impl FollowCrates {
  pub fn follow(&mut self, crate_id: String) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.follow_crate(crate_id.clone());
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }

  pub fn refresh(&mut self, crate_id: String) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.refresh_crate(crate_id.clone());
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }

  pub fn process_update_one(&mut self, response: UpdateOne) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    let krate = response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id, "update crate");
    self.state.id_to_crate.insert(crate_id, krate);

    Ok(())
  }
}


/// Update or set all crates response.
#[derive(Debug)]
pub struct UpdateAll<const SET: bool> {
  result: Result<Vec<Crate>, AttHttpClientError>,
}

impl FollowCrates {
  pub fn get_followed(&mut self) -> impl Future<Output=UpdateAll<true>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.search_crates(CrateSearchQuery::from_followed());
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn refresh_outdated(&mut self) -> impl Future<Output=UpdateAll<false>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.refresh_outdated_crates();
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn refresh_all(&mut self) -> impl Future<Output=UpdateAll<true>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.refresh_all_crates();
    async move {
      UpdateAll { result: future.await }
    }
  }

  pub fn process_update_all<const SET: bool>(&mut self, response: UpdateAll<SET>) -> Result<(), AttHttpClientError> {
    self.all_crates_being_modified = false;

    let crates = response.result
      .inspect_err(|cause| error!(%cause, "failed to update crates: {cause:?}"))?;
    if SET {
      self.state.id_to_crate.clear();
    }
    for krate in crates {
      debug!(crate_id = krate.id, "update crate");
      self.state.id_to_crate.insert(krate.id.clone(), krate);
    }

    Ok(())
  }
}

/// Unfollow crate response.
#[derive(Debug)]
pub struct Unfollow {
  crate_id: String,
  result: Result<(), AttHttpClientError>,
}

impl FollowCrates {
  pub fn unfollow(&mut self, crate_id: String) -> impl Future<Output=Unfollow> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.unfollow_crate(crate_id.clone());
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
    self.state.id_to_crate.swap_remove(&crate_id);

    Ok(())
  }
}


// Service implementation

/// Follow crate requests in message form.
#[derive(Clone, Debug)]
pub enum FollowCrateRequest {
  GetFollowed,
  Follow(String),
  Unfollow(String),
  Refresh(String),
  RefreshOutdated,
  RefreshAll,
}

/// Follow crate responses in message form.
#[derive(Debug)]
pub enum FollowCratesResponse {
  UpdateOne(UpdateOne),
  UpdateAll(UpdateAll<false>),
  SetAll(UpdateAll<true>),
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
impl From<Unfollow> for FollowCratesResponse {
  #[inline]
  fn from(r: Unfollow) -> Self { Self::Unfollow(r) }
}

impl Service for FollowCrates {
  fn action_definitions(&self) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_text("Refresh Outdated"),
      ActionDef::from_text("Refresh All"),
    ];
    ACTION_DEFS
  }

  fn actions(&self) -> impl IntoIterator<Item=impl Action<Request=Self::Request>> {
    let disabled = self.are_all_crates_being_modified();
    [
      ServiceAction { kind: ServiceActionKind::RefreshOutdated, disabled },
      ServiceAction { kind: ServiceActionKind::RefreshAll, disabled },
    ]
  }


  type Data = Crate;

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


  #[inline]
  fn query_definition(&self) -> &QueryDef {
    &self.query_def
  }

  #[inline]
  fn query(&self) -> &Query {
    &self.query
  }

  #[inline]
  fn query_mut(&mut self) -> &mut Query {
    &mut self.query
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
    let crate_id = &data.id;
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
      Follow(crate_id) => self.follow(crate_id).map(UpdateOne).boxed_maybe_send(),
      FollowCrateRequest::Unfollow(crate_id) => self.unfollow(crate_id).map(FollowCratesResponse::Unfollow).boxed_maybe_send(),
      Refresh(crate_id) => self.refresh(crate_id).map(UpdateOne).boxed_maybe_send(),
      RefreshOutdated => self.refresh_outdated().map(UpdateAll).boxed_maybe_send(),
      RefreshAll => self.refresh_all().map(SetAll).boxed_maybe_send(),
    }
  }

  #[inline]
  fn process(&mut self, response: Self::Response) {
    use FollowCratesResponse::*;
    match response {
      UpdateOne(r) => { let _ = self.process_update_one(r); }
      UpdateAll(r) => { let _ = self.process_update_all(r); }
      SetAll(r) => { let _ = self.process_update_all(r); }
      Unfollow(r) => { let _ = self.process_unfollow(r); }
    }
  }
}

// Service actions

enum ServiceActionKind {
  RefreshOutdated,
  RefreshAll,
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
      ServiceActionKind::RefreshOutdated => FollowCrateRequest::RefreshOutdated,
      ServiceActionKind::RefreshAll => FollowCrateRequest::RefreshAll,
    }
  }
}

// Data actions

enum DataActionKind {
  Refresh,
  Unfollow,
}

struct DataAction<'i> {
  kind: DataActionKind,
  disabled: bool,
  crate_id: &'i str,
}

impl Action for DataAction<'_> {
  type Request = FollowCrateRequest;

  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> FollowCrateRequest {
    match self.kind {
      DataActionKind::Refresh => FollowCrateRequest::Refresh(self.crate_id.to_string()),
      DataActionKind::Unfollow => FollowCrateRequest::Unfollow(self.crate_id.to_string()),
    }
  }
}
