use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::future::Future;

use axum::{Json, Router};
use axum::extract::{Path, Query, State};
use axum_login::AuthUser;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use att_core::crates::{Crate, CrateSearch};
use crates_io_client::{CratesIoClient, CratesIoClientError};

use crate::data::Database;
use crate::job_scheduler::{Job, JobAction, JobResult};
use crate::users::AuthSession;
use crate::util::F;

pub mod crates_io_client;

// Data

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct CratesData {
  followed_crate_ids: HashMap<u64, BTreeSet<String>>,
  id_to_crate: BTreeMap<String, (Crate, DateTime<Utc>)>,
}


// Implementation

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
  pub async fn search(&self, search_term: String) -> Result<Vec<Crate>, CratesIoClientError> {
    let response = self.crates_io_client.search(search_term).await?;
    let crates = response.crates.into_iter().map(|c| c.into()).collect();
    Ok(crates)
  }
  pub async fn get_followed_crates(&self, data: &CratesData, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
    let crates = if let Some(followed_crate_ids) = data.followed_crate_ids.get(&user_id) {
      let mut crates = Vec::with_capacity(followed_crate_ids.len());
      for crate_id in followed_crate_ids {
        if let Some((krate, _)) = data.id_to_crate.get(crate_id) {
          crates.push(krate.clone());
        };
      }
      crates
    } else {
      Vec::new()
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

  pub async fn follow(&self, data: &mut CratesData, crate_id: &str, user_id: u64) -> Result<Crate, CratesIoClientError> {
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
    data.followed_crate_ids.entry(user_id).or_default().insert(crate_id.to_string());
    Ok(krate)
  }
  pub fn unfollow(&self, data: &mut CratesData, crate_id: &str, user_id: u64) {
    if let Some(followed_crate_ids) = data.followed_crate_ids.get_mut(&user_id) {
      followed_crate_ids.remove(crate_id);
    }
    // Note: this does not remove the crate from `data.id_to_crate`, as there could be other followers of the crate.
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
    // TODO: remove data from unfollowed crates? Probably best done in a separate step and done in a job.
    let mut refreshed = Vec::new();
    // Refresh outdated cached crate data.
    for (krate, last_refreshed) in data.id_to_crate.values_mut() {
      let crate_id = &krate.id;
      if should_refresh(&now, last_refreshed) {
        let response = self.crates_io_client.refresh(crate_id.clone()).await?;
        *krate = response.crate_data.into();
        *last_refreshed = now;
        refreshed.push(krate.clone());
      }
    }
    // Refresh missing cached crate data.
    for crate_id in data.followed_crate_ids.values().flatten() {
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


// Routing

#[derive(Clone)]
pub struct CratesRoutingState {
  database: Database,
  crates: Crates,
}
impl CratesRoutingState {
  pub fn new(database: Database, crates: Crates) -> Self {
    Self { database, crates }
  }
}
pub fn router() -> Router<CratesRoutingState> {
  use axum::routing::{get, post};
  Router::new()
    .route("/", get(search_crates))
    .route("/:crate_id", get(get_crate))
    .route("/:crate_id/follow", post(follow_crate).delete(unfollow_crate))
    .route("/:crate_id/refresh", post(refresh_crate))
    .route("/refresh_outdated", post(refresh_outdated_crates))
    .route("/refresh_all", post(refresh_all_crates))
}
async fn search_crates(auth_session: AuthSession, State(state): State<CratesRoutingState>, Query(search): Query<CrateSearch>) -> Result<Json<Vec<Crate>>, F> {
  let data = state.database.read().await;
  let crates = match search {
    CrateSearch { followed: true, .. } => {
      if let Some(user) = &auth_session.user {
        state.crates.get_followed_crates(&data.crates, user.id()).await?
      } else {
        return Err(F::unauthorized())
      }
    }
    CrateSearch { search_term: Some(search_term), .. } => state.crates.search(search_term).await?,
    _ => Vec::default()
  };
  Ok(Json(crates))
}
async fn get_crate(State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = state.database.write().await;
  let krate = state.crates.get(&mut data.crates, &crate_id).await?;
  Ok(Json(krate))
}

async fn follow_crate(auth_session: AuthSession, State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let user_id = auth_session.user.ok_or(F::unauthorized())?.id();
  let mut data = state.database.write().await;
  let krate = state.crates.follow(&mut data.crates, &crate_id, user_id).await?;
  Ok(Json(krate))
}
async fn unfollow_crate(auth_session: AuthSession, State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> Result<(), F> {
  let user_id = auth_session.user.ok_or(F::unauthorized())?.id();
  let mut data = state.database.write().await;
  state.crates.unfollow(&mut data.crates, &crate_id, user_id);
  Ok(())
}

async fn refresh_crate(State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = state.database.write().await;
  let krate = state.crates.refresh_one(&mut data.crates, crate_id).await?;
  Ok(Json(krate))
}
async fn refresh_outdated_crates(State(state): State<CratesRoutingState>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = state.database.write().await;
  let crates = state.crates.refresh_outdated(&mut data.crates).await?;
  Ok(Json(crates))
}
async fn refresh_all_crates(State(state): State<CratesRoutingState>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = state.database.write().await;
  let crates = state.crates.refresh_all(&mut data.crates).await?;
  Ok(Json(crates))
}


// Scheduled Jobs

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
