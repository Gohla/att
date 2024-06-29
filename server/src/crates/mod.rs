use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::extract::{Path, Query, State};
use axum::Router;
use axum_login::AuthUser;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use att_core::crates::{Crate, CrateError, CrateSearchQuery};
use crates_io_client::CratesIoClient;

use crate::crates::crates_io_client::CratesIoClientError;
use crate::crates::crates_io_dump::{CratesIoDump, UpdateCratesIoDumpJob};
use crate::data::Database;
use crate::job_scheduler::{Job, JobAction, JobResult};
use crate::users::AuthSession;
use crate::util::JsonResult;

pub mod crates_io_client;
pub mod crates_io_dump;

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
  crates_io_dump: Arc<RwLock<CratesIoDump>>,
}

impl Crates {
  pub fn new(
    crates_io_user_agent: &str,
    crates_io_db_dump_file: PathBuf
  ) -> Result<(Self, impl Future<Output=()>), Box<dyn Error>> {
    let (crates_io_client, task) = CratesIoClient::new(crates_io_user_agent)?;
    let crates_io_dump = Arc::new(RwLock::new(CratesIoDump::new(crates_io_db_dump_file)));
    let crates = Self { crates_io_client, crates_io_dump };
    Ok((crates, task))
  }
}

impl Crates {
  #[instrument(skip(self), err)]
  pub async fn search(&self, search_term: String) -> Result<Option<Vec<Crate>>, CratesIoClientError> {
    let crates = self.crates_io_dump.read().unwrap().crates().postfix_search::<String, _>(&search_term) // TODO: don't block!
      .map(|(_, krate)| krate.clone())
      .collect();
    Ok(Some(crates))
  }

  #[instrument(skip(self, data))]
  pub async fn get_followed_crates(&self, data: &CratesData, user_id: u64) -> Vec<Crate> {
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
    crates
  }

  #[instrument(skip(self, data), err)]
  pub async fn get(&self, data: &mut CratesData, crate_id: String) -> Result<Crate, CratesIoClientError> {
    self.ensure_refreshed(&mut data.id_to_crate, &crate_id, Utc::now(), refresh_hourly).await
  }


  #[instrument(skip(self, data), err)]
  pub async fn follow(&self, data: &mut CratesData, crate_id: String, user_id: u64) -> Result<Crate, CratesIoClientError> {
    let krate = self.ensure_refreshed(&mut data.id_to_crate, &crate_id, Utc::now(), refresh_hourly).await?;
    data.followed_crate_ids.entry(user_id).or_default().insert(crate_id);
    Ok(krate)
  }

  pub fn unfollow(&self, data: &mut CratesData, crate_id: &str, user_id: u64) {
    if let Some(followed_crate_ids) = data.followed_crate_ids.get_mut(&user_id) {
      followed_crate_ids.remove(crate_id);
    }
    // Note: this does not remove the crate from `data.id_to_crate`, as there could be other followers of the crate.
  }


  #[instrument(skip(self, data), err)]
  pub async fn refresh_one(&self, data: &mut CratesData, crate_id: String) -> Result<Crate, CratesIoClientError> {
    self.ensure_refreshed(&mut data.id_to_crate, &crate_id, Utc::now(), |_, _| true).await
  }

  #[instrument(skip(self, data), err)]
  pub async fn refresh_outdated(&self, data: &mut CratesData, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
    self.refresh_multiple(data, user_id, Utc::now(), refresh_hourly).await
  }

  #[instrument(skip(self, data), err)]
  pub async fn refresh_all(&self, data: &mut CratesData, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
    self.refresh_multiple(data, user_id, Utc::now(), |_, _| true).await
  }


  #[instrument(skip_all, err)]
  async fn refresh_for_all_users(
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

  #[instrument(skip_all, err)]
  async fn refresh_multiple(
    &self,
    data: &mut CratesData,
    user_id: u64,
    now: DateTime<Utc>,
    should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
  ) -> Result<Vec<Crate>, CratesIoClientError> {
    let mut refreshed = Vec::new();
    if let Some(followed_crate_ids) = data.followed_crate_ids.get(&user_id) {
      for crate_id in followed_crate_ids {
        let krate = self.ensure_refreshed(&mut data.id_to_crate, crate_id, now, &should_refresh).await?;
        refreshed.push(krate);
      }
    }
    Ok(refreshed)
  }

  async fn ensure_refreshed(
    &self,
    id_to_crate: &mut BTreeMap<String, (Crate, DateTime<Utc>)>,
    crate_id: &String,
    now: DateTime<Utc>,
    should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
  ) -> Result<Crate, CratesIoClientError> {
    let krate = if let Some((krate, last_refreshed)) = id_to_crate.get_mut(crate_id) {
      if should_refresh(&now, last_refreshed) {
        let response = self.crates_io_client.refresh(crate_id.clone()).await?;
        *krate = response.crate_data.into();
        *last_refreshed = now;
      }
      krate.clone()
    } else {
      let response = self.crates_io_client.refresh(crate_id.clone()).await?;
      let krate: Crate = response.crate_data.into();
      id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
      krate
    }; // Note: can't use entry API due to async.
    Ok(krate)
  }
}

fn refresh_hourly(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>) -> bool {
  now.signed_duration_since(last_refresh) > Duration::hours(1)
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

async fn search_crates(
  auth_session: AuthSession,
  State(state): State<CratesRoutingState>,
  Query(search): Query<CrateSearchQuery>
) -> JsonResult<Vec<Crate>, CrateError> {
  async move {
    let data = state.database.read().await;
    let crates = match search {
      CrateSearchQuery { followed: true, .. } => {
        if let Some(user) = &auth_session.user {
          state.crates.get_followed_crates(&data.crates, user.id()).await
        } else {
          return Err(CrateError::NotLoggedIn)
        }
      }
      CrateSearchQuery { search_term: Some(search_term), .. } => state.crates.search(search_term).await
        .map_err(|_| CrateError::Internal)?
        .unwrap_or_default(),
      _ => Vec::default()
    };
    Ok(crates)
  }.await.into()
}

async fn get_crate(State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> JsonResult<Crate, CrateError> {
  async move {
    let mut data = state.database.write().await;
    let krate = state.crates.get(&mut data.crates, crate_id).await
      .map_err(CratesIoClientError::into_crate_error)?;
    Ok(krate)
  }.await.into()
}


async fn follow_crate(auth_session: AuthSession, State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> JsonResult<Crate, CrateError> {
  async move {
    let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
    let mut data = state.database.write().await;
    let krate = state.crates.follow(&mut data.crates, crate_id, user_id).await
      .map_err(CratesIoClientError::into_crate_error)?;
    Ok(krate)
  }.await.into()
}

async fn unfollow_crate(auth_session: AuthSession, State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> JsonResult<(), CrateError> {
  async move {
    let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
    let mut data = state.database.write().await;
    state.crates.unfollow(&mut data.crates, &crate_id, user_id);
    Ok(())
  }.await.into()
}


async fn refresh_crate(auth_session: AuthSession, State(state): State<CratesRoutingState>, Path(crate_id): Path<String>) -> JsonResult<Crate, CrateError> {
  async move {
    let _ = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
    let mut data = state.database.write().await;
    let krate = state.crates.refresh_one(&mut data.crates, crate_id).await
      .map_err(CratesIoClientError::into_crate_error)?;
    Ok(krate)
  }.await.into()
}

async fn refresh_outdated_crates(auth_session: AuthSession, State(state): State<CratesRoutingState>) -> JsonResult<Vec<Crate>, CrateError> {
  async move {
    let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
    let mut data = state.database.write().await;
    let crates = state.crates.refresh_outdated(&mut data.crates, user_id).await
      .map_err(CratesIoClientError::into_crate_error)?;
    Ok(crates)
  }.await.into()
}

async fn refresh_all_crates(auth_session: AuthSession, State(state): State<CratesRoutingState>) -> JsonResult<Vec<Crate>, CrateError> {
  async move {
    let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
    let mut data = state.database.write().await;
    let crates = state.crates.refresh_all(&mut data.crates, user_id).await
      .map_err(CratesIoClientError::into_crate_error)?;
    Ok(crates)
  }.await.into()
}


// Scheduled Jobs

impl Crates {
  pub fn create_update_crates_io_dump_job(&self) -> UpdateCratesIoDumpJob {
    UpdateCratesIoDumpJob::new(self.crates_io_dump.clone())
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
    self.crates.refresh_for_all_users(&mut self.database.write().await.crates, Utc::now(), refresh_hourly).await?;
    Ok(JobAction::Continue)
  }
}
