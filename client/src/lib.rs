use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use att_core::crates::{Crate, CrateSearch};
use att_core::users::UserCredentials;

use crate::http_client::{AttHttpClient, AttHttpClientError};

pub mod http_client;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub id_to_crate: BTreeMap<String, Crate>,
}

pub struct AttClient {
  data: Data,
  http_client: AttHttpClient,
}
impl AttClient {
  #[inline]
  pub fn new(data: Data, http_client: AttHttpClient) -> Self {
    Self { data, http_client }
  }

  #[inline]
  pub fn data(&self) -> &Data { &self.data }

  #[inline]
  pub async fn login(&self, user_credentials: UserCredentials) -> Result<(), AttHttpClientError> {
    self.http_client.clone().login(user_credentials).await
  }

  #[inline]
  pub async fn request_followed_crates(&mut self) -> Result<(), AttHttpClientError> {
    let crates = self.http_client.clone().search_crates(CrateSearch::followed()).await?;
    self.data.id_to_crate.clear();
    for krate in crates {
      self.data.id_to_crate.insert(krate.id.clone(), krate);
    }
    Ok(())
  }

  #[inline]
  pub async fn follow_crate(&mut self, crate_id: String) -> Result<(), AttHttpClientError> {
    let krate = self.http_client.clone().follow_crate(crate_id).await?;
    self.data.id_to_crate.insert(krate.id.clone(), krate);
    Ok(())
  }
  #[inline]
  pub async fn unfollow_crate(&mut self, crate_id: String) -> Result<(), AttHttpClientError> {
    self.http_client.clone().unfollow_crate(crate_id.clone()).await?;
    self.data.id_to_crate.remove(&crate_id);
    Ok(())
  }

  #[inline]
  pub async fn refresh_crate(&mut self, crate_id: String) -> Result<(), AttHttpClientError> {
    let krate = self.http_client.clone().refresh_crate(crate_id).await?;
    self.data.id_to_crate.insert(krate.id.clone(), krate);
    Ok(())
  }
  #[inline]
  pub async fn refresh_outdated_crates(&mut self) -> Result<(), AttHttpClientError> {
    let crates = self.http_client.clone().refresh_outdated_crates().await?;
    for krate in crates {
      self.data.id_to_crate.insert(krate.id.clone(), krate);
    }
    Ok(())
  }
  #[inline]
  pub async fn refresh_all_crates(&mut self) -> Result<(), AttHttpClientError> {
    let crates = self.http_client.clone().refresh_all_crates().await?;
    self.data.id_to_crate.clear();
    for krate in crates {
      self.data.id_to_crate.insert(krate.id.clone(), krate);
    }
    Ok(())
  }
}
