use std::future::Future;

use reqwest::{Method, RequestBuilder};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tracing::{debug, instrument};
use url::Url;

use att_core::crates::{CrateError, CrateSearchQuery, FullCrate};
use att_core::users::{AuthError, UserCredentials};

#[derive(Clone, Debug)]
pub struct AttHttpClient {
  http_client: reqwest::Client,
  base_url: Url,
}

impl AttHttpClient {
  pub fn new(http_client: reqwest::Client, base_url: Url) -> Self {
    Self { http_client, base_url }
  }
  pub fn from_base_url(base_url: impl reqwest::IntoUrl) -> Result<Self, reqwest::Error> {
    #[cfg(not(target_arch = "wasm32"))] let http_client = {
      reqwest::Client::builder()
        .cookie_store(true)
        .build()?
    };
    #[cfg(target_arch = "wasm32")] let http_client = {
      reqwest::Client::builder()
        .build()?
    };
    let base_url = base_url.into_url()?;
    Ok(Self::new(http_client, base_url))
  }
}

#[derive(Debug, Error)]
pub enum AttHttpClientError {
  #[error("HTTP request failed")]
  Request(#[from] reqwest::Error),
  #[error("Users request failed")]
  Login(#[from] AuthError),
  #[error("Crate request failed")]
  Crate(#[from] CrateError),
}

impl AttHttpClient {
  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  pub fn login(&self, user_credentials: UserCredentials) -> impl Future<Output=Result<(), AttHttpClientError>> {
    let rb = self.request_builder(Method::POST, "users/login")
      .json(&user_credentials);
    async move { Self::send::<_, AuthError>(rb).await }
  }
  #[instrument(skip_all, err)]
  pub fn logout(&self) -> impl Future<Output=Result<(), AttHttpClientError>> {
    let rb = self.request_builder(Method::DELETE, "users/login");
    async move { Self::send::<_, AuthError>(rb).await }
  }

  #[instrument(skip(self), err)]
  pub fn search_crates(&self, crate_search: CrateSearchQuery) -> impl Future<Output=Result<Vec<FullCrate>, AttHttpClientError>> {
    let rb = self.request_builder(Method::GET, "crates")
      .query(&crate_search);
    async move { Self::send::<_, CrateError>(rb).await }
  }

  #[instrument(skip(self), err)]
  pub fn follow_crate(&self, crate_id: i32) -> impl Future<Output=Result<(), AttHttpClientError>> {
    let rb = self.request_builder(Method::POST, format!("crates/{crate_id}/follow"));
    async move { Self::send::<_, CrateError>(rb).await }
  }
  #[instrument(skip(self), err)]
  pub fn unfollow_crate(&self, crate_id: i32) -> impl Future<Output=Result<(), AttHttpClientError>> {
    let rb = self.request_builder(Method::DELETE, format!("crates/{crate_id}/follow"));
    async move { Self::send::<_, CrateError>(rb).await }
  }

  #[instrument(skip(self), err)]
  pub fn refresh_crate(&self, crate_id: i32) -> impl Future<Output=Result<FullCrate, AttHttpClientError>> {
    let rb = self.request_builder(Method::POST, format!("crates/{crate_id}/refresh"));
    async move { Self::send::<_, CrateError>(rb).await }
  }
  #[instrument(skip(self), err)]
  pub fn refresh_followed(&self) -> impl Future<Output=Result<Vec<FullCrate>, AttHttpClientError>> {
    let rb = self.request_builder(Method::POST, "crates/refresh_followed");
    async move { Self::send::<_, CrateError>(rb).await }
  }

  fn request_builder(&self, method: Method, join_url: impl AsRef<str>) -> RequestBuilder {
    let url = self.base_url.join(join_url.as_ref()).expect("BUG: creating URL failed");
    let request_builder = self.http_client.request(method, url);
    #[cfg(not(target_arch = "wasm32"))] {
      request_builder
    }
    #[cfg(target_arch = "wasm32")] {
      use wasm_bindgen::JsCast;

      let cookie = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .dyn_into::<web_sys::HtmlDocument>()
        .unwrap()
        .cookie()
        .unwrap();

      request_builder
        .header("cookie", cookie)
        .fetch_credentials_include()
    }
  }
  async fn send<T: DeserializeOwned, E: DeserializeOwned>(
    request_builder: RequestBuilder,
  ) -> Result<T, AttHttpClientError> where
    AttHttpClientError: From<E>
  {
    debug!(request = ?request_builder, "sending HTTP request");
    let response = request_builder.send().await?;
    let body: Result<T, E> = response.json().await?;
    Ok(body?)
  }
}
