use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use att_core::Crate;

use crate::krate::crates_io_client::CratesIoClient;

pub mod crates_io_client;

#[derive(Default, Serialize, Deserialize)]
pub struct CrateData {
  blessed_crate_ids: BTreeSet<String>,
  #[serde(default)]
  id_to_crate: BTreeMap<String, (Crate, DateTime<Utc>)>
}
impl CrateData {
  pub fn blessed_crates(&self) -> Vec<Crate> {
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

  pub async fn refresh(&mut self, id: String, crates_io_client: &CratesIoClient) -> Result<Crate, Box<dyn Error>> {
    let response = crates_io_client.clone().refresh(id.clone()).await??;
    let krate: Crate = response.crate_data.into();
    self.id_to_crate.insert(id, (krate.clone(), Utc::now()));
    Ok(krate)
  }
  pub async fn refresh_outdated_data(&mut self, crates_io_client: &CratesIoClient) -> Result<(), Box<dyn Error>> {
    let now = Utc::now();
    let should_refresh = |last_refresh: &_| now.signed_duration_since(last_refresh) > Duration::hours(1);
    self.refresh_data(crates_io_client, now, should_refresh).await
  }
  pub async fn refresh_all_data(&mut self, crates_io_client: &CratesIoClient) -> Result<(), Box<dyn Error>> {
    self.refresh_data(crates_io_client, Utc::now(), |_| true).await
  }
  async fn refresh_data(
    &mut self,
    crates_io_client: &CratesIoClient,
    now: DateTime<Utc>,
    should_refresh: impl Fn(&DateTime<Utc>) -> bool
  ) -> Result<(), Box<dyn Error>> {
    // Refresh outdated cached crate data.
    for (krate, last_refreshed) in self.id_to_crate.values_mut() {
      let id = &krate.id;
      if self.blessed_crate_ids.contains(id) {
        if should_refresh(last_refreshed) {
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
}
