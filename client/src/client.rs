
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

  pub async fn get_blessed_crates(self) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crate/blessed")?;
    let request = self.http_client.get(url).build()?;
    let response = self.http_client.execute(request).await?;
    let blessed_crates: Vec<Crate> = response.json().await?;
    Ok(blessed_crates)
  }
  pub async fn add_blessed_crate(self, id: String) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crate/blessed/add")?;
    let request = self.http_client.get(url).build()?;
    let response = self.http_client.execute(request).await?;
    let blessed_crates: Vec<Crate> = response.json().await?;
    Ok(blessed_crates)
  }

  pub async fn crate_search(self, search: Search) -> Result<Vec<Crate>, ClientError> {
    let url = self.base_url.join("crate/search")?;
    let request = self.http_client.post(url).json(&search).build()?;
    let response = self.http_client.execute(request).await?;
    let blessed_crates: Vec<Crate> = response.json().await?;
    Ok(blessed_crates)
  }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
  #[error("Parsing URL failed")]
  UrlParse(#[from] url::ParseError),
  #[error("HTTP request failed")]
  Request(#[from] reqwest::Error)
}
