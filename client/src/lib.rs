use serde::{Deserialize, Serialize};

use app::AppViewData;
use crates::{CrateData, CrateViewData};

use crate::app::AppRequest;
use crate::crates::CrateRequest;
use crate::http_client::AttHttpClient;

pub mod http_client;
pub mod app;
pub mod crates;

/// Client data: the local view of the data that is on the server. Should be (de)serialized between runs of the program.
#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  crates: CrateData,
}
impl Data {
  #[inline]
  pub fn crates(&self) -> &CrateData { &self.crates }
  #[inline]
  pub fn crates_mut(&mut self) -> &mut CrateData { &mut self.crates }
}

/// Client view data: the runtime data needed to properly view data, request updates from the server, send
/// modifications to the server, and to apply operations that update the [data](Data) and this [view data](ViewData).
#[derive(Default)]
pub struct ViewData {
  app: AppViewData,
  crates: CrateViewData,
}
impl ViewData {
  #[inline]
  pub fn app(&self) -> &AppViewData { &self.app }
  #[inline]
  pub fn app_mut(&mut self) -> &mut AppViewData { &mut self.app }

  #[inline]
  pub fn crates(&self) -> &CrateViewData { &self.crates }
  #[inline]
  pub fn crates_mut(&mut self) -> &mut CrateViewData { &mut self.crates }
}


/// Client to asynchronously request updates and send updates to the server. All async methods return an operation
/// that must be applied to the [data](Data) and [view data](ViewData) when the future completes.
#[derive(Clone)]
pub struct AttClient {
  http_client: AttHttpClient,
}
impl AttClient {
  #[inline]
  pub fn new(http_client: AttHttpClient) -> Self { Self { http_client } }
  pub fn from_base_url(base_url: impl reqwest::IntoUrl) -> Result<Self, reqwest::Error> {
    Ok(Self::new(AttHttpClient::from_base_url(base_url)?))
  }

  #[inline]
  pub fn app(&self) -> AppRequest { AppRequest::new(self.http_client.clone()) }
  #[inline]
  pub fn crates(&self) -> CrateRequest { CrateRequest::new(self.http_client.clone()) }

  #[inline]
  pub fn http_client(&self) -> &AttHttpClient { &self.http_client }
}
