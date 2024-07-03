use std::error::Error;
use std::future::Future;
use std::path::PathBuf;

use thiserror::Error;
use tracing::instrument;

use att_core::crates::Crate;
use att_server_db::{DbError, DbPool};
use att_server_db::crates::{CratesDb, UpdateCrate, UpdateDownloads};
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


// crates.io API refresh

#[derive(Debug, Error)]
pub enum InternalError {
  #[error("crates.io API operation failed: {0}")]
  CratesIoClient(#[from] CratesIoClientError),
  #[error("Database operation failed: {0}")]
  Database(#[from] DbError),
}


impl Crates {
  #[instrument(skip(self), err)]
  pub async fn refresh_one(&self, crate_id: i32) -> Result<Option<Crate>, InternalError> {
    if let Some(crate_name) = self.db_pool.query(move |db| db.find_name(crate_id)).await? {
      let response = self.crates_io_client.refresh(crate_name).await?;
      let update_crate = UpdateCrate { // TODO: update more fields
        id: crate_id,
        updated_at: Some(response.crate_data.updated_at),
        description: response.crate_data.description,
        homepage: Some(response.crate_data.homepage),
        repository: Some(response.crate_data.repository),
        ..UpdateCrate::default()
      };
      let update_downloads = UpdateDownloads {
        crate_id,
        downloads: response.crate_data.downloads as i64,
      };
      let (krate, _downloads) = self.db_pool.query(move |db|{
        let krate = db.update_crate(update_crate)?;
        let downloads = db.update_crate_downloads(update_downloads)?;
        Ok((krate, downloads))
      }).await?;
      Ok(krate)
    } else {
      Ok(None)
    }
  }

  #[instrument(skip(self), err)]
  pub async fn refresh_all(&self, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
    todo!()
  }
}
// impl Crates {
//   #[instrument(skip(self), err)]
//   pub async fn refresh_one(&self, crate_id: String) -> Result<Crate, CratesIoClientError> {
//     self.ensure_refreshed(&mut data.id_to_crate, &crate_id, Utc::now(), |_, _| true).await
//   }
//
//   #[instrument(skip(self), err)]
//   pub async fn refresh_outdated(&self, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
//     self.refresh_multiple(data, user_id, Utc::now(), refresh_hourly).await
//   }
//
//   #[instrument(skip(self), err)]
//   pub async fn refresh_all(&self, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
//     self.refresh_multiple(data, user_id, Utc::now(), |_, _| true).await
//   }
//
//
//   #[instrument(skip_all, err)]
//   async fn refresh_for_all_users(
//     &self,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Vec<Crate>, CratesIoClientError> {
//     // TODO: remove data from unfollowed crates? Probably best done in a separate step and done in a job.
//     let mut refreshed = Vec::new();
//     // Refresh outdated cached crate data.
//     for (krate, last_refreshed) in data.id_to_crate.values_mut() {
//       let crate_id = &krate.name;
//       if should_refresh(&now, last_refreshed) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         *krate = response.crate_data.into();
//         *last_refreshed = now;
//         refreshed.push(krate.clone());
//       }
//     }
//     // Refresh missing cached crate data.
//     for crate_id in data.followed_crate_ids.values().flatten() {
//       if !data.id_to_crate.contains_key(crate_id) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         let krate: Crate = response.crate_data.into();
//         data.id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
//         refreshed.push(krate);
//       }
//     }
//     Ok(refreshed)
//   }
//
//   #[instrument(skip_all, err)]
//   async fn refresh_multiple(
//     &self,
//     user_id: u64,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Vec<Crate>, CratesIoClientError> {
//     let mut refreshed = Vec::new();
//     if let Some(followed_crate_ids) = data.followed_crate_ids.get(&user_id) {
//       for crate_id in followed_crate_ids {
//         let krate = self.ensure_refreshed(&mut data.id_to_crate, crate_id, now, &should_refresh).await?;
//         refreshed.push(krate);
//       }
//     }
//     Ok(refreshed)
//   }
//
//   async fn ensure_refreshed(
//     &self,
//     id_to_crate: &mut BTreeMap<String, (Crate, DateTime<Utc>)>,
//     crate_id: &String,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Crate, CratesIoClientError> {
//     let krate = if let Some((krate, last_refreshed)) = id_to_crate.get_mut(crate_id) {
//       if should_refresh(&now, last_refreshed) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         *krate = response.crate_data.into();
//         *last_refreshed = now;
//       }
//       krate.clone()
//     } else {
//       let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//       let krate: Crate = response.crate_data.into();
//       id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
//       krate
//     }; // Note: can't use entry API due to async.
//     Ok(krate)
//   }
// }
//
// fn refresh_hourly(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>) -> bool {
//   now.signed_duration_since(last_refresh) > Duration::hours(1)
// }
