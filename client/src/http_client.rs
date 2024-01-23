use reqwest::{Method, RequestBuilder};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tracing::{debug, instrument};
use url::Url;

use att_core::crates::{Crate, CrateError, CrateSearch};
use att_core::users::{UserCredentials, UsersError};

#[derive(Clone)]
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
  #[error("Parsing URL failed")]
  UrlParse(#[from] url::ParseError),
  #[error("HTTP request failed")]
  Request(#[from] reqwest::Error),
  #[error("Users request failed")]
  Login(#[from] UsersError),
  #[error("Crate request failed")]
  Crate(#[from] CrateError),
}

impl AttHttpClient {
  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  pub async fn login(self, user_credentials: UserCredentials) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join("users/login")?;
    debug!(?user_credentials, url = url.to_string(), "sending login request");
    self.request::<_, UsersError>(Method::POST, url, |b| b.json(&user_credentials)).await
  }
  #[instrument(skip_all, err)]
  pub async fn logout(self) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join("users/login")?;
    debug!(%url, "sending logout request");
    self.request::<_, UsersError>(Method::DELETE, url, |b| b).await
  }

  #[instrument(skip(self), err)]
  pub async fn search_crates(self, crate_search: CrateSearch) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates")?;
    debug!(?crate_search, %url, "sending search crates request");
    self.request::<_, CrateError>(Method::GET, url, |b| b.query(&crate_search)).await
  }

  #[instrument(skip(self), err)]
  pub async fn follow_crate(self, crate_id: String) -> Result<Crate, AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/follow"))?;
    debug!(crate_id, %url, "sending follow crate request");
    self.request::<_, CrateError>(Method::POST, url, |b| b).await
  }
  #[instrument(skip(self), err)]
  pub async fn unfollow_crate(self, crate_id: String) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/follow"))?;
    debug!(crate_id, %url, "sending unfollow crate request");
    self.request::<_, CrateError>(Method::DELETE, url, |b| b).await
  }

  #[instrument(skip(self), err)]
  pub async fn refresh_crate(self, crate_id: String) -> Result<Crate, AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/refresh"))?;
    debug!(crate_id, %url, "sending refresh crate request");
    self.request::<_, CrateError>(Method::POST, url, |b| b).await
  }
  #[instrument(skip(self), err)]
  pub async fn refresh_outdated_crates(self) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates/refresh_outdated")?;
    debug!(%url, "sending refresh outdated crates request");
    self.request::<_, CrateError>(Method::POST, url, |b| b).await
  }
  #[instrument(skip(self), err)]
  pub async fn refresh_all_crates(self) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates/refresh_all")?;
    debug!(%url, "sending refresh all crates request");
    self.request::<_, CrateError>(Method::POST, url, |b| b).await
  }

  async fn request<T: DeserializeOwned, E: DeserializeOwned>(
    &self,
    method: Method,
    url: Url,
    modify_request: impl FnOnce(RequestBuilder) -> RequestBuilder,
  ) -> Result<T, AttHttpClientError> where AttHttpClientError: From<E> {
    let request_builder = self.http_client.request(method, url).set_cookie_wasm();
    let request_builder = modify_request(request_builder);
    let response = request_builder.send().await?;
    let body: Result<T, E> = response.json().await?;
    Ok(body?)
  }
}

trait RequestBuilderExt {
  fn set_cookie_wasm(self) -> Self;
}
#[cfg(not(target_arch = "wasm32"))]
impl RequestBuilderExt for RequestBuilder {
  fn set_cookie_wasm(self) -> Self { self /* Do nothing */ }
}
#[cfg(target_arch = "wasm32")]
impl RequestBuilderExt for RequestBuilder {
  fn set_cookie_wasm(self) -> Self {
    use wasm_bindgen::JsCast;

    let cookie = web_sys::window()
      .unwrap()
      .document()
      .unwrap()
      .dyn_into::<web_sys::HtmlDocument>()
      .unwrap()
      .cookie()
      .unwrap();

    self
      .header("cookie", cookie)
      .fetch_credentials_include()
  }
}
