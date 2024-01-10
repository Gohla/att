use reqwest::RequestBuilder;
use thiserror::Error;
use tracing::{debug, instrument};
use url::Url;

use att_core::crates::{Crate, CrateSearch};
use att_core::users::UserCredentials;

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
  Request(#[from] reqwest::Error)
}

impl AttHttpClient {
  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  pub async fn login(self, user_credentials: UserCredentials) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join("users/login")?;
    debug!(?user_credentials, url = url.to_string(), "sending login request");
    let request_builder = self.http_client.post(url).json(&user_credentials);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    self.http_client.execute(request).await?.error_for_status()?;

    Ok(())
  }
  #[instrument(skip_all, err)]
  pub async fn logout(self) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join("users/login")?;
    debug!(%url, "sending logout request");
    let request_builder = self.http_client.delete(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    self.http_client.execute(request).await?.error_for_status()?;
    Ok(())
  }

  #[instrument(skip(self), err)]
  pub async fn search_crates(self, crate_search: CrateSearch) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates")?;
    debug!(?crate_search, %url, "sending search crates request");
    let request_builder = self.http_client.get(url).query(&crate_search);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    let response = self.http_client.execute(request).await?.error_for_status()?;
    let crates = response.json().await?;
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn follow_crate(self, crate_id: String) -> Result<Crate, AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/follow"))?;
    debug!(crate_id, %url, "sending follow crate request");
    let request_builder = self.http_client.post(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    let response = self.http_client.execute(request).await?.error_for_status()?;
    let krate = response.json().await?;
    Ok(krate)
  }
  #[instrument(skip(self), err)]
  pub async fn unfollow_crate(self, crate_id: String) -> Result<(), AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/follow"))?;
    debug!(crate_id, %url, "sending unfollow crate request");
    let request_builder = self.http_client.delete(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    self.http_client.execute(request).await?.error_for_status()?;
    Ok(())
  }

  #[instrument(skip(self), err)]
  pub async fn refresh_crate(self, crate_id: String) -> Result<Crate, AttHttpClientError> {
    let url = self.base_url.join(&format!("crates/{crate_id}/refresh"))?;
    debug!(crate_id, %url, "sending refresh crate request");
    let request_builder = self.http_client.post(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    let response = self.http_client.execute(request).await?.error_for_status()?;
    let krate = response.json().await?;
    Ok(krate)
  }
  #[instrument(skip(self), err)]
  pub async fn refresh_outdated_crates(self) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates/refresh_outdated")?;
    debug!(%url, "sending refresh outdated crates request");
    let request_builder = self.http_client.post(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    let response = self.http_client.execute(request).await?.error_for_status()?;
    let crates: Vec<Crate> = response.json().await?;
    Ok(crates)
  }
  #[instrument(skip(self), err)]
  pub async fn refresh_all_crates(self) -> Result<Vec<Crate>, AttHttpClientError> {
    let url = self.base_url.join("crates/refresh_all")?;
    debug!(%url, "sending refresh all crates request");
    let request_builder = self.http_client.post(url);
    let request_builder = Self::set_cookie_wasm(request_builder);
    let request = request_builder.build()?;

    let response = self.http_client.execute(request).await?.error_for_status()?;
    let crates: Vec<Crate> = response.json().await?;
    Ok(crates)
  }


  #[cfg(not(target_arch = "wasm32"))]
  fn set_cookie_wasm(request_builder: RequestBuilder) -> RequestBuilder {
    request_builder
  }
  #[cfg(target_arch = "wasm32")]
  fn set_cookie_wasm(request_builder: RequestBuilder) -> RequestBuilder {
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
