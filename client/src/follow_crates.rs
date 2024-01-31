use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

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
  pub fn is_any_crate_being_modified(&self) -> bool {
    self.all_crates_being_modified || !self.crates_being_modified.is_empty()
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
#[derive(Debug)]
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
