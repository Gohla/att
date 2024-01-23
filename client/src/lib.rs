use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use att_core::crates::{Crate, CrateSearch};
use att_core::users::UserCredentials;

use crate::http_client::{AttHttpClient, AttHttpClientError};

pub mod http_client;

/// Client data: the local view of the data that is on the server. Should be (de)serialized between runs of the program.
#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub id_to_crate: BTreeMap<String, Crate>,
}

/// Client view data: the runtime data needed to properly view data, request updates from the server, send
/// modifications to the server, and to apply operations that update the [data](Data) and this [view data](ViewData).
#[derive(Default)]
pub struct ViewData {
  logging_in_or_out: bool,
  logged_in: bool,

  crates_being_modified: BTreeSet<String>,
  all_crates_being_modified: bool,
}
impl ViewData {
  #[inline]
  pub fn logging_in_or_out(&self) -> bool { self.logging_in_or_out }
  #[inline]
  pub fn logged_in(&self) -> bool { self.logged_in }

  #[inline]
  pub fn crates_being_modified(&self) -> &BTreeSet<String> { &self.crates_being_modified }
  #[inline]
  pub fn all_crates_being_modified(&self) -> bool { self.all_crates_being_modified }
  #[inline]
  pub fn is_crate_being_modified(&self, crate_id: &str) -> bool {
    self.all_crates_being_modified || self.crates_being_modified.contains(crate_id)
  }
  #[inline]
  pub fn is_any_crate_being_modified(&self) -> bool {
    self.all_crates_being_modified || !self.crates_being_modified.is_empty()
  }
}

/// Client to asynchronously request updates and send updates to the server. All async methods return an operation
/// that must be applied to the [data](Data) and [view data](ViewData) when the future completes.
#[derive(Clone)]
pub struct AttClient {
  http_client: AttHttpClient,
}
impl AttClient {
  pub fn new(http_client: AttHttpClient) -> Self {
    Self { http_client }
  }
  pub fn from_base_url(base_url: impl reqwest::IntoUrl) -> Result<Self, reqwest::Error> {
    Ok(Self::new(AttHttpClient::from_base_url(base_url)?))
  }

  #[inline]
  pub fn http_client(&self) -> &AttHttpClient { &self.http_client }


  #[inline]
  pub fn login(self, view_data: &mut ViewData, user_credentials: UserCredentials) -> impl Future<Output=Login> {
    view_data.logging_in_or_out = true;
    async move {
      let result = self.http_client.login(user_credentials).await;
      Login { result }
    }
  }
  #[inline]
  pub fn logout(self, view_data: &mut ViewData) -> impl Future<Output=Logout> {
    view_data.logging_in_or_out = true;
    async move {
      let result = self.http_client.logout().await;
      Logout { result }
    }
  }

  #[inline]
  pub fn get_followed_crates(self, view_data: &mut ViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.search_crates(CrateSearch::followed()).await;
      UpdateCrates { result }
    }
  }
  #[inline]
  pub fn follow_crate(self, view_data: &mut ViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.follow_crate(crate_id.clone()).await;
      UpdateCrate { crate_id, result }
    }
  }
  #[inline]
  pub fn unfollow_crate(self, view_data: &mut ViewData, crate_id: String) -> impl Future<Output=RemoveCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.unfollow_crate(crate_id.clone()).await;
      RemoveCrate { crate_id, result }
    }
  }

  #[inline]
  pub fn refresh_outdated_crates(self, view_data: &mut ViewData) -> impl Future<Output=UpdateCrates<false>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.refresh_outdated_crates().await;
      UpdateCrates { result }
    }
  }
  #[inline]
  pub fn refresh_all_crates(self, view_data: &mut ViewData) -> impl Future<Output=UpdateCrates<true>> {
    view_data.all_crates_being_modified = true;
    async move {
      let result = self.http_client.refresh_all_crates().await;
      UpdateCrates { result }
    }
  }
  #[inline]
  pub fn refresh_crate(self, view_data: &mut ViewData, crate_id: String) -> impl Future<Output=UpdateCrate> {
    view_data.crates_being_modified.insert(crate_id.clone());
    async move {
      let result = self.http_client.refresh_crate(crate_id.clone()).await;
      UpdateCrate { crate_id, result }
    }
  }
}


// Operations

#[derive(Debug)]
pub struct Login {
  result: Result<(), AttHttpClientError>,
}
impl Login {
  pub fn apply(self, view_data: &mut ViewData) -> Result<(), AttHttpClientError> {
    view_data.logging_in_or_out = false;

    self.result
      .inspect_err(|cause| error!(%cause, "failed to login: {cause:?}"))?;
    debug!("logged in");
    view_data.logged_in = true;

    Ok(())
  }
}
#[derive(Debug)]
pub struct Logout {
  result: Result<(), AttHttpClientError>,
}
impl Logout {
  pub fn apply(self, view_data: &mut ViewData) -> Result<(), AttHttpClientError> {
    view_data.logging_in_or_out = false;

    self.result
      .inspect_err(|cause| error!(%cause, "failed to logout: {cause:?}"))?;
    debug!("logged out");
    view_data.logged_in = false;

    Ok(())
  }
}

#[derive(Debug)]
pub struct UpdateCrates<const SET_CRATES: bool> {
  result: Result<Vec<Crate>, AttHttpClientError>,
}
impl<const SET_CRATES: bool> UpdateCrates<SET_CRATES> {
  pub fn apply(self, data: &mut Data, view_data: &mut ViewData) -> Result<(), AttHttpClientError> {
    view_data.all_crates_being_modified = false;

    let crates = self.result
      .inspect_err(|cause| error!(%cause, "failed to update crates: {cause:?}"))?;
    if SET_CRATES {
      data.id_to_crate.clear();
    }
    for krate in crates {
      debug!(crate_id = krate.id, "update crate");
      data.id_to_crate.insert(krate.id.clone(), krate);
    }

    Ok(())
  }
}
#[derive(Debug)]
pub struct UpdateCrate {
  crate_id: String,
  result: Result<Crate, AttHttpClientError>,
}
impl UpdateCrate {
  pub fn apply(self, data: &mut Data, view_data: &mut ViewData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    let krate = self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to update crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "update crate");
    data.id_to_crate.insert(self.crate_id, krate);

    Ok(())
  }
}
#[derive(Debug)]
pub struct RemoveCrate {
  crate_id: String,
  result: Result<(), AttHttpClientError>,
}
impl RemoveCrate {
  pub fn apply(self, data: &mut Data, view_data: &mut ViewData) -> Result<(), AttHttpClientError> {
    view_data.crates_being_modified.remove(&self.crate_id);

    self.result
      .inspect_err(|cause| error!(crate_id = self.crate_id, %cause, "failed to remove crate: {cause:?}"))?;
    debug!(crate_id = self.crate_id, "remove crate");
    data.id_to_crate.remove(&self.crate_id);

    Ok(())
  }
}
