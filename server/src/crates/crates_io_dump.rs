use std::error::Error;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use db_dump::Loader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures::StreamExt;
use tokio::fs;
use tokio::fs::File;
use tokio::task::block_in_place;
use tracing::{info, instrument};

use att_core::crates::{Crate, CrateDefaultVersion, CrateDownloads, CrateVersion};

use crate::data::DbPool;
use crate::job_scheduler::{Job, JobAction, JobResult};

#[derive(Clone)]
pub struct CratesIoDump {
  db_dump_file: PathBuf,
  db_pool: DbPool,
}

impl CratesIoDump {
  pub fn new(db_dump_file: PathBuf, db_pool: DbPool) -> Self {
    Self { db_dump_file, db_pool }
  }
}


// Internals

pub const UPDATE_DURATION: Duration = Duration::from_secs(60 * 60 * 24);

impl CratesIoDump {
  #[instrument(skip_all, err)]
  async fn import_db_dump(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    info!("Reading database dump");

    let mut all_crates = Vec::new();
    let mut all_crate_downloads = Vec::new();
    let mut all_crate_versions = Vec::new();
    let mut all_crate_default_versions = Vec::new();

    block_in_place(|| Loader::new()
      .crates(|row| {
        all_crates.push(Crate {
          id: row.id.0 as i32,
          name: row.name,
          updated_at: row.updated_at,
          created_at: row.created_at,
          description: row.description,
          homepage: row.homepage,
          readme: row.readme,
          repository: row.repository,
        });
      })
      .crate_downloads(|row| {
        all_crate_downloads.push(CrateDownloads {
          crate_id: row.crate_id.0 as i32,
          downloads: row.downloads as i64,
        });
      })
      .versions(|row| {
        all_crate_versions.push(CrateVersion {
          id: row.id.0 as i32,
          crate_id: row.crate_id.0 as i32,
          version: row.num.to_string(),
        });
      })
      .default_versions(|row| {
        all_crate_default_versions.push(CrateDefaultVersion {
          crate_id: row.crate_id.0 as i32,
          version_id: row.version_id.0 as i32,
        });
      })
      .load(&self.db_dump_file)
    )?;

    info!("Importing database dump");

    // TODO: transaction
    let mut conn = self.db_pool.get().await?;
    use diesel::{insert_into, upsert::excluded};
    let crates = {
      use att_core::schema::crates::dsl::*;
      insert_into(crates)
        .values(&all_crates)
        .on_conflict(id).do_update().set(id.eq(excluded(id))) // From: https://stackoverflow.com/questions/47626047/execute-an-insert-or-update-using-diesel#comment82217514_47626103
        .execute(&mut conn).await?
    };
    let crate_downloads = {
      use att_core::schema::crate_downloads::dsl::*;
      insert_into(crate_downloads)
        .values(&all_crate_downloads)
        .on_conflict(crate_id).do_update().set(crate_id.eq(excluded(crate_id)))
        .execute(&mut conn).await?
    };
    let crate_versions = {
      use att_core::schema::crate_versions::dsl::*;
      insert_into(crate_versions)
        .values(&all_crate_versions)
        .on_conflict(id).do_update().set(id.eq(excluded(id)))
        .execute(&mut conn).await?
    };
    let crate_default_versions = {
      use att_core::schema::crate_default_versions::dsl::*;
      insert_into(crate_default_versions)
        .values(&all_crate_default_versions)
        .on_conflict(crate_id).do_update().set(crate_id.eq(excluded(crate_id)))
        .execute(&mut conn).await?
    };
    let metadata = {
      use att_core::schema::import_crates_metadata::dsl::*;
      insert_into(import_crates_metadata)
        .values(imported_at.eq(Utc::now()))
        .execute(&mut conn).await?
    };

    info!(crates, crate_downloads, crate_versions, crate_default_versions, metadata, "Imported database dump");
    Ok(())
  }

  #[instrument(skip_all, err)]
  async fn is_import_required(&self) -> Result<bool, Box<dyn Error + Send + Sync + 'static>> {
    use att_core::schema::import_crates_metadata::dsl::*;
    let mut conn = self.db_pool.get().await?;
    let import_count: i64 = import_crates_metadata.count().get_result(&mut conn).await?;
    Ok(import_count == 0)
  }

  #[instrument(skip_all, err)]
  fn update_db_dump_file(&self) -> impl Future<Output=Result<bool, Box<dyn Error + Send + Sync + 'static>>> {
    let db_dump_file = self.db_dump_file.clone();

    async move {
      let is_up_to_date = match fs::metadata(&db_dump_file).await {
        Ok(metadata) => metadata.modified()?.elapsed()? < UPDATE_DURATION,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => Err(e)?,
      };
      if is_up_to_date {
        return Ok(false)
      }

      const URL: &str = "https://static.crates.io/db-dump.tar.gz";
      info!("Downloading crates.io database dump '{}' into '{}'", URL, db_dump_file.display());

      if let Some(parent) = db_dump_file.parent() {
        fs::create_dir_all(parent).await?;
      }
      let mut file = File::create(&db_dump_file).await?;

      let response = reqwest::get(URL).await?;
      let mut bytes_stream = response.bytes_stream();

      while let Some(bytes) = bytes_stream.next().await {
        let bytes = bytes?;
        tokio::io::copy(&mut bytes.as_ref(), &mut file).await?;
      }
      Ok(true)
    }
  }
}


// Scheduled job

pub struct UpdateCratesIoDumpJob {
  crates_io_dump: CratesIoDump,
}

impl UpdateCratesIoDumpJob {
  pub fn new(crates_io_dump: CratesIoDump) -> Self {
    Self { crates_io_dump }
  }
}

impl Job for UpdateCratesIoDumpJob {
  async fn run(&self) -> JobResult {
    let db_dump_file_updated = self.crates_io_dump.update_db_dump_file().await?;
    let import_required = self.crates_io_dump.is_import_required().await?;
    if db_dump_file_updated || import_required {
      self.crates_io_dump.import_db_dump().await?;
    }
    Ok(JobAction::Continue)
  }
}
