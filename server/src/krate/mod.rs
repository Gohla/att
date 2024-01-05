use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use att_core::{Crate, Search};

use crate::data::Database;
use crate::job_scheduler::{Job, JobAction, JobResult};
use crate::krate::crates_io_client::{CratesIoClient, CratesIoClientError};

pub mod crates_io_client;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct CratesData {
  followed_crate_ids: BTreeSet<String>,
  #[serde(default)]
  id_to_crate: BTreeMap<String, (Crate, DateTime<Utc>)>,
}

#[derive(Clone)]
pub struct Crates {
  crates_io_client: CratesIoClient,
}
impl Crates {
  pub fn new(user_agent: &str) -> Result<(Self, impl Future<Output=()>), Box<dyn Error>> {
    let (crates_io_client, task) = CratesIoClient::new(user_agent)?;
    Ok((Self { crates_io_client }, task))
  }
}

impl Crates {
  pub async fn search(&self, data: &mut CratesData, search: Search) -> Result<Vec<Crate>, CratesIoClientError> {
    let crates = match search {
      Search { followed: true, .. } => {
        let mut crates = Vec::with_capacity(data.followed_crate_ids.len());
        for crate_id in &data.followed_crate_ids {
          if let Some((krate, _)) = data.id_to_crate.get(crate_id) {
            crates.push(krate.clone());
          };
        }
        crates
      }
      Search { search_term: Some(search_term), .. } => {
        let response = self.crates_io_client.search(search_term).await?;
        let now = Utc::now();
        for krate in &response.crates {
          let krate: Crate = krate.into();
          data.id_to_crate.insert(krate.id.clone(), (krate, now));
        }
        let crates = response.crates.into_iter().map(|c| c.into()).collect();
        crates
      }
      _ => {
        data.id_to_crate.values().map(|(krate, _)| krate.clone()).collect()
      }
    };
    Ok(crates)
  }
  pub async fn get(&self, data: &mut CratesData, crate_id: &str) -> Result<Crate, CratesIoClientError> {
    let krate = if let Some((krate, _)) = data.id_to_crate.get(crate_id) {
      krate.clone()
    } else {
      self.refresh_one(data, crate_id.to_string()).await?
    };
    Ok(krate)
  }

  pub async fn follow(&self, data: &mut CratesData, crate_id: &str) -> Result<Crate, CratesIoClientError> {
    let now = Utc::now();
    let krate = if let Some((krate, last_refreshed)) = data.id_to_crate.get_mut(crate_id) {
      if Self::should_refresh(&now, last_refreshed) {
        let response = self.crates_io_client.refresh(crate_id.to_string()).await?;
        *krate = response.crate_data.into();
        *last_refreshed = now;
      }
      krate.clone()
    } else {
      self.refresh_one(data, crate_id.to_string()).await?
    };
    data.followed_crate_ids.insert(crate_id.to_string());
    Ok(krate)
  }
  pub fn unfollow(&self, data: &mut CratesData, id: String) {
    data.followed_crate_ids.remove(&id);
    data.id_to_crate.remove(&id);
  }

  pub async fn refresh_one(&self, data: &mut CratesData, crate_id: String) -> Result<Crate, CratesIoClientError> {
    let response = self.crates_io_client.refresh(crate_id).await?;
    let krate: Crate = response.crate_data.into();
    let now = Utc::now();
    data.id_to_crate.insert(krate.id.clone(), (krate.clone(), now));
    Ok(krate)
  }
  pub async fn refresh_outdated(&self, data: &mut CratesData) -> Result<Vec<Crate>, CratesIoClientError> {
    self.refresh_multiple(data, Utc::now(), Self::should_refresh).await
  }
  pub async fn refresh_all(&self, data: &mut CratesData) -> Result<Vec<Crate>, CratesIoClientError> {
    self.refresh_multiple(data, Utc::now(), |_, _| true).await
  }

  async fn refresh_multiple(
    &self,
    data: &mut CratesData,
    now: DateTime<Utc>,
    should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
  ) -> Result<Vec<Crate>, CratesIoClientError> {
    let mut refreshed = Vec::new();
    // Refresh outdated cached crate data.
    for (krate, last_refreshed) in data.id_to_crate.values_mut() {
      let crate_id = &krate.id;
      if data.followed_crate_ids.contains(crate_id) {
        if should_refresh(&now, last_refreshed) {
          let response = self.crates_io_client.refresh(crate_id.clone()).await?;
          *krate = response.crate_data.into();
          *last_refreshed = now;
          refreshed.push(krate.clone());
        }
      }
    }
    // Refresh missing cached crate data.
    for crate_id in &data.followed_crate_ids {
      if !data.id_to_crate.contains_key(crate_id) {
        let response = self.crates_io_client.refresh(crate_id.clone()).await?;
        let krate: Crate = response.crate_data.into();
        data.id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
        refreshed.push(krate);
      }
    }
    Ok(refreshed)
  }
  fn should_refresh(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>) -> bool {
    now.signed_duration_since(last_refresh) > Duration::hours(1)
  }
}

pub struct RefreshJob {
  crates: Crates,
  database: Database,
}
impl RefreshJob {
  pub fn new(crates: Crates, database: Database) -> Self {
    Self { crates, database }
  }
}
impl Job for RefreshJob {
  async fn run(&self) -> JobResult {
    self.crates.refresh_outdated(&mut self.database.write().await.crates).await?;
    Ok(JobAction::Continue)
  }
}
