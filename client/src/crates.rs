use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{Crate, CrateSearch};
use att_core::util::maybe_send::{MaybeSend, MaybeSendFutureExt};

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


/// Crate requests.
#[derive(Clone)]
pub struct CrateRequest {
  http_client: AttHttpClient,
}
impl CrateRequest {
  #[inline]
  pub(crate) fn new(http_client: AttHttpClient) -> Self { Self { http_client } }

  pub fn get_followed(self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.search_crates(CrateSearch::followed()).await;
      UpdateCrates { result }
    }
  }
  pub fn follow(self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.follow_crate(crate_id.clone()).await;
      UpdateCrate { crate_id, result }
    }
  }
  pub fn unfollow(self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=RemoveCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.unfollow_crate(crate_id.clone()).await;
      RemoveCrate { crate_id, result }
    }
  }

  pub fn refresh_outdated(self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<false>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.refresh_outdated_crates().await;
      UpdateCrates { result }
    }
  }
  pub fn refresh_all(self, view_data: &mut CrateViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.refresh_all_crates().await;
      UpdateCrates { result }
    }
  }
  pub fn refresh(self, view_data: &mut CrateViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.refresh_crate(crate_id.clone()).await;
      UpdateCrate { crate_id, result }
    }
  }
}

/// Crate action: request in message form.
#[derive(Debug)]
pub enum CrateAction {
  GetFollowed,
  Follow(String),
  Unfollow(String),
  Refresh(String),
  RefreshOutdated,
  RefreshAll,
}
impl CrateAction {
  pub fn perform(self, request: CrateRequest, view_data: &mut CrateViewData) -> impl Future<Output=CrateOperation> + MaybeSend + 'static {
    use CrateAction::*;
    use CrateOperation::*;
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


/// Update single crate operation.
#[derive(Debug)]
pub struct UpdateCrate {
  crate_id: String,
  result: Result<Crate, AttHttpClientError>,
}
impl UpdateCrate {
  pub fn apply(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    let krate = self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "update crate");
    data.id_to_crate.insert(self.crate_id, krate);

    Ok(())
  }
}

/// Update/set multiple crates operation.
#[derive(Debug)]
pub struct UpdateCrates<const SET: bool> {
  result: Result<Vec<Crate>, AttHttpClientError>,
}
impl<const SET: bool> UpdateCrates<SET> {
  pub fn apply(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
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

/// Remove (unfollow) crate operation.
#[derive(Debug)]
pub struct RemoveCrate {
  crate_id: String,
  result: Result<(), AttHttpClientError>,
}
impl RemoveCrate {
  pub fn apply(self, view_data: &mut CrateViewData, data: &mut CrateData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to remove crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "remove crate");
    data.id_to_crate.remove(&self.crate_id);

    Ok(())
  }
}

/// Crate operation in message form.
#[derive(Debug)]
pub enum CrateOperation {
  UpdateOne(UpdateCrate),
  Update(UpdateCrates<false>),
  Set(UpdateCrates<true>),
  Remove(RemoveCrate),
}
impl From<UpdateCrate> for CrateOperation {
  #[inline]
  fn from(op: UpdateCrate) -> Self { Self::UpdateOne(op) }
}
impl From<UpdateCrates<false>> for CrateOperation {
  #[inline]
  fn from(op: UpdateCrates<false>) -> Self { Self::Update(op) }
}
impl From<UpdateCrates<true>> for CrateOperation {
  #[inline]
  fn from(op: UpdateCrates<true>) -> Self { Self::Set(op) }
}
impl From<RemoveCrate> for CrateOperation {
  #[inline]
  fn from(op: RemoveCrate) -> Self { Self::Remove(op) }
}
impl CrateOperation {
  pub fn apply(self, view_data: &mut CrateViewData, data: &mut CrateData) {
    match self {
      CrateOperation::UpdateOne(operation) => { let _ = operation.apply(view_data, data); }
      CrateOperation::Update(operation) => { let _ = operation.apply(view_data, data); }
      CrateOperation::Set(operation) => { let _ = operation.apply(view_data, data); }
      CrateOperation::Remove(operation) => { let _ = operation.apply(view_data, data); }
    }
  }
}
