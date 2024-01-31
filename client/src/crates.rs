use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{Crate, CrateSearchQuery};
use att_core::util::maybe_send::MaybeSendFuture;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Crate data.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CrateData {
  id_to_crate: BTreeMap<String, Crate>,
}
impl CrateData {
  #[inline]
  pub fn num_followed_crates(&self) -> usize { self.id_to_crate.len() }
  #[inline]
  pub fn followed_crates(&self) -> impl Iterator<Item=&Crate> { self.id_to_crate.values() }
}

/// Crate view data.
#[derive(Default, Debug)]
pub struct CrateViewData {
  crates_being_modified: BTreeSet<String>,
  all_crates_being_modified: bool,
}
impl CrateViewData {
  #[inline]
  pub fn is_crate_being_modified(&self, crate_id: &str) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(crate_id)
  }
  #[inline]
  pub fn is_any_crate_being_modified(&self) -> bool {
    self.all_crates_being_modified || !self.crates_being_modified.is_empty()
  }
}


/// Crate client for requesting (changes to) data.
#[derive(Clone)]
pub struct CrateClient {
  http_client: AttHttpClient,
}
impl CrateClient {
  #[inline]
  pub(crate) fn new(http_client: AttHttpClient) -> Self { Self { http_client } }

  pub fn get_followed(&self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    let future = self.http_client.search_crates(CrateSearchQuery::from_followed());
    async move {
      UpdateCrates { result: future.await }
    }
  }
  pub fn follow(&self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.follow_crate(crate_id.clone());
    async move {
      UpdateCrate { crate_id, result: future.await }
    }
  }
  pub fn unfollow(&self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=RemoveCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.unfollow_crate(crate_id.clone());
    async move {
      RemoveCrate { crate_id, result: future.await }
    }
  }

  pub fn refresh_outdated(&self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<false>> {
    view_data.all_crates_being_modified = true;
    let future = self.http_client.refresh_outdated_crates();
    async move {
      UpdateCrates { result: future.await }
    }
  }
  pub fn refresh_all(&self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    let future = self.http_client.refresh_all_crates();
    async move {
      UpdateCrates { result: future.await }
    }
  }
  pub fn refresh(&self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    let future = self.http_client.refresh_crate(crate_id.clone());
    async move {
      UpdateCrate { crate_id, result: future.await }
    }
  }
}

/// Crate requests in message form.
#[derive(Debug)]
pub enum CrateRequest {
  GetFollowed,
  Follow(String),
  Unfollow(String),
  Refresh(String),
  RefreshOutdated,
  RefreshAll,
}
impl CrateRequest {
  pub fn send(self, request: &CrateClient, view_data: &mut CrateViewData) -> impl MaybeSendFuture<'static, Output=CrateResponse> {
    use CrateRequest::*;
    use CrateResponse::*;
    match self {
      GetFollowed => request.get_followed(view_data).map(Set).boxed_maybe_send(),
      Follow(crate_id) => request.follow(view_data, crate_id).map(UpdateOne).boxed_maybe_send(),
      Unfollow(crate_id) => request.unfollow(view_data, crate_id).map(Remove).boxed_maybe_send(),
      Refresh(crate_id) => request.refresh(view_data, crate_id).map(UpdateOne).boxed_maybe_send(),
      RefreshOutdated => request.refresh_outdated(view_data).map(Update).boxed_maybe_send(),
      RefreshAll => request.refresh_all(view_data).map(Set).boxed_maybe_send(),
    }
  }
}


/// Update single crate response.
#[derive(Debug)]
pub struct UpdateCrate {
  crate_id: String,
  result: Result<Crate, AttHttpClientError>,
}
impl UpdateCrate {
  pub fn process(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    let krate = self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "update crate");
    data.id_to_crate.insert(self.crate_id, krate);

    Ok(())
  }
}

/// Update/set multiple crates response.
#[derive(Debug)]
pub struct UpdateCrates<const SET: bool> {
  result: Result<Vec<Crate>, AttHttpClientError>,
}
impl<const SET: bool> UpdateCrates<SET> {
  pub fn process(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
    view_data.all_crates_being_modified = false;

    let crates = self.result
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
}

/// Remove (unfollow) crate response.
#[derive(Debug)]
pub struct RemoveCrate {
  crate_id: String,
  result: Result<(), AttHttpClientError>,
}
impl RemoveCrate {
  pub fn process(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to remove crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "remove crate");
    data.id_to_crate.remove(&self.crate_id);

    Ok(())
  }
}

/// Crate responses in message form.
#[derive(Debug)]
pub enum CrateResponse {
  UpdateOne(UpdateCrate),
  Update(UpdateCrates<false>),
  Set(UpdateCrates<true>),
  Remove(RemoveCrate),
}
impl From<UpdateCrate> for CrateResponse {
  #[inline]
  fn from(r: UpdateCrate) -> Self { Self::UpdateOne(r) }
}
impl From<UpdateCrates<false>> for CrateResponse {
  #[inline]
  fn from(r: UpdateCrates<false>) -> Self { Self::Update(r) }
}
impl From<UpdateCrates<true>> for CrateResponse {
  #[inline]
  fn from(r: UpdateCrates<true>) -> Self { Self::Set(r) }
}
impl From<RemoveCrate> for CrateResponse {
  #[inline]
  fn from(r: RemoveCrate) -> Self { Self::Remove(r) }
}
impl CrateResponse {
  pub fn process(self, view_data: &mut CrateViewData, data: &mut CrateData) {
    match self {
      CrateResponse::UpdateOne(r) => { let _ = r.process(view_data, data); }
      CrateResponse::Update(r) => { let _ = r.process(view_data, data); }
      CrateResponse::Set(r) => { let _ = r.process(view_data, data); }
      CrateResponse::Remove(r) => { let _ = r.process(view_data, data); }
    }
  }
}
