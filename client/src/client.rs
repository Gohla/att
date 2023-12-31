use reqwest::{Client as HttpClient, IntoUrl};
use url::Url;

use att_core::{Crate, Search};

#[derive(Clone)]
pub struct Client {
  http_client: HttpClient,
  base_url: Url,
}
impl Client {
  pub fn new(http_client: HttpClient, base_url: Url) -> Self {
    Self { http_client, base_url }
  }
  pub fn from_base_url(base_url: impl IntoUrl) -> Result<Self, ClientError> {
    let http_client = HttpClient::builder().build()?;
    let base_url = base_url.into_url()?;
    Ok(Self::new(http_client, base_url))
  }

  pub async fn search_crates(self, search: Search) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crates")?;
    let request = self.http_client.get(url).json(&search).build()?;
    let response = self.http_client.execute(request).await?;
    let crates = response.json().await?;
    Ok(crates)
  }

  pub async fn follow_crate(self, crate_id: String) -> Result<Crate, ClientError> {
    let url = self.base_url.join("crates")?.join(&crate_id)?.join("follow")?;
    let request = self.http_client.post(url).build()?;
    let response = self.http_client.execute(request).await?;
    let krate = response.json().await?;
    Ok(krate)
  }
  pub async fn unfollow_crate(self, crate_id: String) -> Result<(), ClientError> {
    let url = self.base_url.join("crates")?.join(&crate_id)?.join("follow")?;
    let request = self.http_client.delete(url).build()?;
    self.http_client.execute(request).await?;
    Ok(())
  }

  pub async fn refresh_crate(self, crate_id: String) -> Result<Crate, ClientError> {
    let url = self.base_url.join("crates")?.join(&crate_id)?.join("refresh")?;
    let request = self.http_client.post(url).build()?;
    let response = self.http_client.execute(request).await?;
    let krate = response.json().await?;
    Ok(krate)
  }
  pub async fn refresh_outdated_crates(self) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crates/refresh_outdated")?;
    let request = self.http_client.post(url).build()?;
    let response = self.http_client.execute(request).await?;
    let crates: Vec<Crate> = response.json().await?;
    Ok(crates)
  }
  pub async fn refresh_all_crates(self) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crates/refresh_all")?;
    let request = self.http_client.post(url).build()?;
    let response = self.http_client.execute(request).await?;
    let crates: Vec<Crate> = response.json().await?;
    Ok(crates)
  }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
  #[error("Parsing URL failed")]
  UrlParse(#[from] url::ParseError),
  #[error("HTTP request failed")]
  Request(#[from] reqwest::Error)
}
