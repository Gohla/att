use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use att_core::{Crate, Search};

use crate::krate::crates_io_client::CratesIoClient;

pub mod crates_io_client;

#[derive(Default, Serialize, Deserialize)]
pub struct CrateData {
  blessed_crate_ids: BTreeSet<String>,
  #[serde(default)]
  id_to_crate: BTreeMap<String, (Crate, DateTime<Utc>)>
}
impl CrateData {
  pub fn get_blessed_crates(&self) -> Vec<Crate> {
    let mut crates = Vec::with_capacity(self.blessed_crate_ids.len());
    for id in &self.blessed_crate_ids {
      if let Some((c, _)) = self.id_to_crate.get(id) {
        crates.push(c.clone());
      } else {
        crates.push(Crate::from_id(id.clone()));
      }
    }
    crates
  }
  pub async fn add_blessed_crate(&mut self, id: String, crates_io_client: &CratesIoClient) -> Result<Crate, Box<dyn Error>> {
    let now = Utc::now();
    let krate = if let Some((krate, last_refreshed)) = self.id_to_crate.get_mut(&id) {
      if Self::should_refresh(&now, last_refreshed) {
        let response = crates_io_client.clone().refresh(id.clone()).await??;
        *krate = response.crate_data.into();
        *last_refreshed = now;
      }
      krate.clone()
    } else {
      let response = crates_io_client.clone().refresh(id.clone()).await??;
      let krate: Crate = response.crate_data.into();
      self.id_to_crate.insert(krate.id.clone(), (krate.clone(), now));
      krate
    };
    self.blessed_crate_ids.insert(id);
    Ok(krate)
  }
  pub fn remove_blessed_crate(&mut self, id: String) {
    self.blessed_crate_ids.remove(&id);
    self.id_to_crate.remove(&id);
  }

  pub async fn search(&mut self, search: Search, crates_io_client: &CratesIoClient) -> Result<Vec<Crate>, Box<dyn Error>> {
    let response = crates_io_client.clone().search(search.search_term).await??;
    let now = Utc::now();
    for krate in &response.crates {
      self.update(krate.into(), now);
    }
    let crates = response.crates.into_iter().map(|c| c.into()).collect();
    Ok(crates)
  }

  pub async fn refresh_one(&mut self, id: String, crates_io_client: &CratesIoClient) -> Result<Crate, Box<dyn Error>> {
    let response = crates_io_client.clone().refresh(id.clone()).await??;
    let krate: Crate = response.crate_data.into();
    self.update(krate.clone(), Utc::now());
    Ok(krate)
  }
  pub async fn refresh_outdated(&mut self, crates_io_client: &CratesIoClient) -> Result<(), Box<dyn Error>> {
    self.refresh_multiple(crates_io_client, Utc::now(), Self::should_refresh).await
  }
  pub async fn refresh_all(&mut self, crates_io_client: &CratesIoClient) -> Result<(), Box<dyn Error>> {
    self.refresh_multiple(crates_io_client, Utc::now(), |_, _| true).await
  }
  async fn refresh_multiple(
    &mut self,
    crates_io_client: &CratesIoClient,
    now: DateTime<Utc>,
    should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
  ) -> Result<(), Box<dyn Error>> {
    // Refresh outdated cached crate data.
    for (krate, last_refreshed) in self.id_to_crate.values_mut() {
      let id = &krate.id;
      if self.blessed_crate_ids.contains(id) {
        if should_refresh(&now, last_refreshed) {
          let response = crates_io_client.clone().refresh(id.clone()).await??;
          *krate = response.crate_data.into();
          *last_refreshed = now;
        }
      }
    }
    // Refresh missing cached crate data.
    for id in &self.blessed_crate_ids {
      if !self.id_to_crate.contains_key(id) {
        let response = crates_io_client.clone().refresh(id.clone()).await??;
        self.id_to_crate.insert(id.clone(), (response.crate_data.into(), now));
      }
    }
    Ok(())
  }

  fn update(&mut self, krate: Crate, now: DateTime<Utc>) {
    self.id_to_crate.insert(krate.id.clone(), (krate, now));
  }
  fn should_refresh(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>) -> bool {
    now.signed_duration_since(last_refresh) > Duration::hours(1)
  }
}
