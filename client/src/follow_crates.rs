use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::collection::{Action, ActionDef, Collection};
use att_core::crates::{Crate, CrateSearchQuery};
use att_core::util::maybe_send::MaybeSendFuture;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Follow crates data.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FollowCratesData {
  id_to_crate: BTreeMap<String, Crate>,
}
impl FollowCratesData {
  #[inline]
  pub fn num_followed_crates(&self) -> usize { self.id_to_crate.len() }
  #[inline]
  pub fn followed_crates(&self) -> impl Iterator<Item=&Crate> { self.id_to_crate.values() }
}

/// Keep track of followed crates.
#[derive(Debug)]
pub struct FollowCrates {
  http_client: AttHttpClient,
  crates_being_modified: BTreeSet<String>,
  all_crates_being_modified: bool,
}
impl FollowCrates {
  #[inline]
  pub fn new(http_client: AttHttpClient) -> Self {
    Self {
      http_client,
      crates_being_modified: Default::default(),
      all_crates_being_modified: false,
    }
  }

  #[inline]
  pub fn is_crate_being_modified(&self, crate_id: &str) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(crate_id)
  }
  #[inline]
  pub fn are_all_crates_being_modified(&self) -> bool {
    self.all_crates_being_modified
  }

  pub fn get_followed(&mut self) -> impl Future<Output=UpdateAll<true>> {
    self.all_crates_being_modified = true;
    let future = self.http_client.search_crates(CrateSearchQuery::from_followed());
    async move {
      UpdateAll { result: future.await }
    }
  }
  pub fn follow(&mut self, crate_id: String) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.follow_crate(crate_id.clone());
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }
  pub fn unfollow(&mut self, crate_id: String) -> impl Future<Output=Unfollow> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.unfollow_crate(crate_id.clone());
    async move {
      Unfollow { crate_id, result: future.await }
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
  pub fn refresh(&mut self, crate_id: String) -> impl Future<Output=UpdateOne> {
    self.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.refresh_crate(crate_id.clone());
    async move {
      UpdateOne { crate_id, result: future.await }
    }
  }

  pub fn process_update_one(&mut self, response: UpdateOne, data: &mut FollowCratesData) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    let krate = response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id, "update crate");
    data.id_to_crate.insert(crate_id, krate);

    Ok(())
  }
  pub fn process_update_all<const SET: bool>(&mut self, response: UpdateAll<SET>, data: &mut FollowCratesData) -> Result<(), AttHttpClientError> {
    self.all_crates_being_modified = false;

    let crates = response.result
      .inspect_err(|cause| error!(%cause, "failed to update crates: {cause:?}"))?;
    if SET {
      data.id_to_crate.clear();
    }
    for krate in crates {
      debug!(crate_id = krate.id, "update crate");
      data.id_to_crate.insert(krate.id.clone(), krate);
    }

    Ok(())
  }
  pub fn process_unfollow(&mut self, response: Unfollow, data: &mut FollowCratesData) -> Result<(), AttHttpClientError> {
    let crate_id = response.crate_id;
    self.crates_being_modified.remove(&crate_id);

    response.result
      .inspect_err(|cause| error!(crate_id, %cause, "failed to unfollow crate: {cause:?}"))?;
    debug!(crate_id, "unfollow crate");
    data.id_to_crate.remove(&crate_id);

    Ok(())
  }
}

/// Update one crate response.
#[derive(Debug)]
pub struct UpdateOne {
  crate_id: String,
  result: Result<Crate, AttHttpClientError>,
}

/// Update or set all crates response.
#[derive(Debug)]
pub struct UpdateAll<const SET: bool> {
  result: Result<Vec<Crate>, AttHttpClientError>,
}

/// Unfollow crate response.
#[derive(Debug)]
pub struct Unfollow {
  crate_id: String,
  result: Result<(), AttHttpClientError>,
}


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
impl FollowCrates {
  pub fn send(&mut self, request: FollowCrateRequest) -> impl MaybeSendFuture<'static, Output=FollowCratesResponse> {
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
impl FollowCrates {
  pub fn process(&mut self, response: FollowCratesResponse, data: &mut FollowCratesData) {
    match response {
      FollowCratesResponse::UpdateOne(r) => { let _ = self.process_update_one(r, data); }
      FollowCratesResponse::UpdateAll(r) => { let _ = self.process_update_all(r, data); }
      FollowCratesResponse::SetAll(r) => { let _ = self.process_update_all(r, data); }
      FollowCratesResponse::Unfollow(r) => { let _ = self.process_unfollow(r, data); }
    }
  }
}


enum CollectionActionKind {
  RefreshOutdated,
  RefreshAll,
}

struct CollectionAction {
  kind: CollectionActionKind,
  disabled: bool,
}

impl Action<FollowCrateRequest> for CollectionAction {
  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> FollowCrateRequest {
    match self.kind {
      CollectionActionKind::RefreshOutdated => FollowCrateRequest::RefreshOutdated,
      CollectionActionKind::RefreshAll => FollowCrateRequest::RefreshAll,
    }
  }
}


enum ItemActionKind {
  Refresh,
  Unfollow,
}

struct ItemAction<'i> {
  kind: ItemActionKind,
  disabled: bool,
  crate_id: &'i str,
}

impl Action<FollowCrateRequest> for ItemAction<'_> {
  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> FollowCrateRequest {
    match self.kind {
      ItemActionKind::Refresh => FollowCrateRequest::Refresh(self.crate_id.to_string()),
      ItemActionKind::Unfollow => FollowCrateRequest::Unfollow(self.crate_id.to_string()),
    }
  }
}


impl Collection for FollowCrates {
  fn action_definitions(&self) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_text("Refresh Outdated"),
      ActionDef::from_text("Refresh All"),
    ];
    ACTION_DEFS
  }

  fn actions(&self) -> impl IntoIterator<Item=impl Action<Self::Request>> {
    let disabled = self.are_all_crates_being_modified();
    [
      CollectionAction { kind: CollectionActionKind::RefreshOutdated, disabled },
      CollectionAction { kind: CollectionActionKind::RefreshAll, disabled },
    ]
  }


  type Item = Crate;

  #[inline]
  fn item_action_definitions(&self) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_text("\u{F116}"),
      ActionDef::from_text("\u{F5DE}").with_danger_style(),
    ];
    ACTION_DEFS
  }

  #[inline]
  fn item_action<'i>(&self, index: usize, data: &'i Self::Item) -> Option<impl Action<Self::Request> + 'i> {
    let crate_id = &data.id;
    let disabled = self.is_crate_being_modified(crate_id);
    let action = match index {
      0 => ItemAction { kind: ItemActionKind::Refresh, disabled, crate_id },
      1 => ItemAction { kind: ItemActionKind::Unfollow, disabled, crate_id },
      _ => return None,
    };
    Some(action)
  }


  type Request = FollowCrateRequest;

  type Response = FollowCratesResponse;

  #[inline]
  fn send(&mut self, request: Self::Request) -> impl MaybeSendFuture<'static, Output=Self::Response> {
    self.send(request)
  }

  type Data = FollowCratesData;

  #[inline]
  fn process(&mut self, data: &mut Self::Data, response: Self::Response) {
    self.process(response, data)
  }
}
