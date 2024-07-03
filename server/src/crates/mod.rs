use std::error::Error;
use std::future::Future;
use std::path::PathBuf;

use thiserror::Error;
use tracing::instrument;

use att_core::crates::{CrateError, CrateSearchQuery, FullCrate};
use att_server_db::{DbError, DbPool, DbPoolObj};
use att_server_db::crates::{CratesDb, UpdateCrate};
use crates_io_client::CratesIoClient;

use crate::crates::crates_io_client::CratesIoClientError;
use crate::crates::crates_io_dump::{CratesIoDump, UpdateCratesIoDumpJob};

pub mod crates_io_client;
pub mod crates_io_dump;
pub mod route;

#[derive(Clone)]
pub struct Crates {
  db_pool: DbPool<CratesDb>,
  crates_io_client: CratesIoClient,
  crates_io_dump: CratesIoDump,
}

impl Crates {
  pub fn new(
    db_pool: DbPool,
    crates_io_user_agent: &str,
    crates_io_db_dump_file: PathBuf
  ) -> Result<(Self, impl Future<Output=()>), Box<dyn Error>> {
    let db_pool = db_pool.with();
    let (crates_io_client, task) = CratesIoClient::new(crates_io_user_agent)?;
    let crates_io_dump = CratesIoDump::new(crates_io_db_dump_file, db_pool.clone());
    let crates = Self { db_pool, crates_io_client, crates_io_dump };
    Ok((crates, task))
  }

  pub fn create_update_crates_io_dump_job(&self) -> UpdateCratesIoDumpJob {
    UpdateCratesIoDumpJob::new(self.crates_io_dump.clone())
  }
}


#[derive(Debug, Error)]
pub enum InternalError {
  #[error("Crate with ID {0} was not found")]
  CrateNotFound(i32),
  #[error("crates.io API operation failed: {0}")]
  CratesIoClient(#[from] CratesIoClientError),
  #[error("Database operation failed: {0}")]
  Database(#[from] DbError),
}
impl From<InternalError> for CrateError {
  fn from(e: InternalError) -> Self {
    match e {
      InternalError::CrateNotFound(_) => CrateError::NotFound,
      _ => CrateError::Internal,
    }
  }
}

impl Crates {
  #[instrument(skip(self), err)]
  pub async fn find(&self, crate_id: i32) -> Result<FullCrate, InternalError> {
    self.db_pool.perform(move |conn| conn.find(crate_id))
      .await?
      .ok_or_else(|| InternalError::CrateNotFound(crate_id))
  }

  #[instrument(skip(self), err)]
  pub async fn search(&self, query: CrateSearchQuery, user_id: i32) -> Result<Vec<FullCrate>, InternalError> {
    let crates = match query {
      CrateSearchQuery { followed: true, .. } => self.db_pool
        .query(move |conn| conn.get_followed_crates(user_id))
        .await?,
      CrateSearchQuery { search_term: Some(search_term), .. } => self.db_pool
        .perform(move |conn| conn.search(&search_term))
        .await?,
      _ => Vec::default()
    };
    Ok(crates.into())
  }

  #[instrument(skip(self), err)]
  pub async fn refresh_one(&self, crate_id: i32) -> Result<FullCrate, InternalError> {
    let db_pool_obj = self.db_pool.get().await?;

    let mut full_crate = db_pool_obj.query(move |conn| conn.find(crate_id))
      .await?
      .ok_or_else(|| InternalError::CrateNotFound(crate_id))?;

    self.update(&mut full_crate, &db_pool_obj)
      .await?;

    Ok(full_crate)
  }

  #[instrument(skip(self), err)]
  pub async fn refresh_followed(&self, user_id: i32) -> Result<Vec<FullCrate>, InternalError> {
    let db_pool_obj = self.db_pool.get().await?;

    let mut full_crates = db_pool_obj.query(move |conn| conn.get_followed_crates(user_id))
      .await?;

    for full_crate in &mut full_crates {
      self.update(full_crate, &db_pool_obj)
        .await?;
    }

    Ok(full_crates)
  }

  async fn update(&self, full_crate: &mut FullCrate, db_pool_obj: &DbPoolObj<CratesDb>) -> Result<(), InternalError> {
    let crate_id = full_crate.krate.id;

    let response = self.crates_io_client.refresh(full_crate.krate.name.clone()).await?;
    let update_crate = UpdateCrate { // TODO: update more fields
      id: crate_id,
      updated_at: Some(response.crate_data.updated_at),
      description: response.crate_data.description,
      homepage: Some(response.crate_data.homepage),
      repository: Some(response.crate_data.repository),
      readme: None, // Not in `CrateResponse`.
      downloads: Some(response.crate_data.downloads as i64),
      ..UpdateCrate::default()
    };
    // TODO: update versions and default version

    let krate = db_pool_obj.perform::<InternalError, _>(move |conn| {
      let krate = conn.update_crate(update_crate)?
        .ok_or_else(|| InternalError::CrateNotFound(crate_id))?;
      Ok(krate)
    }).await?;
    full_crate.krate = krate;

    Ok(())
  }
}
