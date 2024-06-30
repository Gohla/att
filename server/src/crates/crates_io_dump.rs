use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTimeError};

use chrono::Utc;
use db_dump::Loader;
use diesel::copy_from;
use diesel::prelude::*;
use futures::StreamExt;
use thiserror::Error;
use tokio::fs;
use tokio::fs::File;
use tokio::task::block_in_place;
use tracing::{info, instrument};

use att_core::crates::{Crate, CrateDefaultVersion, CrateDownloads, CrateVersion};

use crate::data::{DatabaseError, DbPool};
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

pub const UPDATE_DURATION: Duration = Duration::from_secs(60 * 60 * 24);

// Internals

#[derive(Debug, Error)]
pub enum InternalError {
  #[error(transparent)]
  DbDump(#[from] db_dump::Error),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Time(#[from] SystemTimeError),
  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
  #[error(transparent)]
  Database(DatabaseError),
}
impl<E: Into<DatabaseError>> From<E> for InternalError {
  fn from(value: E) -> Self {
    Self::Database(value.into())
  }
}


impl CratesIoDump {
  #[instrument(skip_all, err)]
  async fn import_db_dump(&self) -> Result<(), InternalError> {
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
    // Excluded trick from: https://stackoverflow.com/questions/47626047/execute-an-insert-or-update-using-diesel#comment82217514_47626103

    let conn = self.db_pool.get().await?;
    // use diesel::{insert_into, upsert::excluded};
    use diesel::{insert_into};
    let crates = {
      conn.interact(move |conn| {
        use att_core::schema::crates::dsl::*;
        copy_from(crates)
          .from_insertable(&all_crates)
          .execute(conn)
        // insert_into(crates)
        //   .values(&all_crates)
        //   .on_conflict(id).do_update().set(id.eq(excluded(id)))
        //   .execute(conn)
      }).await??
    };
    let crate_downloads = {
      conn.interact(move |conn| {
        use att_core::schema::crate_downloads::dsl::*;
        copy_from(crate_downloads)
          .from_insertable(&all_crate_downloads)
          .execute(conn)
        // insert_into(crate_downloads)
        //   .values(&all_crate_downloads)
        //   .on_conflict(crate_id).do_update().set(crate_id.eq(excluded(crate_id)))
        //   .execute(conn)
      }).await??
    };
    let crate_versions = {
      conn.interact(move|conn| {
        use att_core::schema::crate_versions::dsl::*;
        copy_from(crate_versions)
          .from_insertable(&all_crate_versions)
          .execute(conn)
        // insert_into(crate_versions)
        //   .values(&all_crate_versions)
        //   .on_conflict(id).do_update().set(id.eq(excluded(id)))
        //   .execute(conn)
      }).await??
    };
    let crate_default_versions = {
      conn.interact(move|conn| {
        use att_core::schema::crate_default_versions::dsl::*;
        copy_from(crate_default_versions)
          .from_insertable(&all_crate_default_versions)
          .execute(conn)
        // insert_into(crate_default_versions)
        //   .values(&all_crate_default_versions)
        //   .on_conflict(crate_id).do_update().set(crate_id.eq(excluded(crate_id)))
        //   .execute(conn)
      }).await??
    };
    let metadata = {
      conn.interact(|conn| {
        use att_core::schema::import_crates_metadata::dsl::*;
        insert_into(import_crates_metadata)
          .values(imported_at.eq(Utc::now()))
          .execute(conn)
      }).await??
    };

    info!(crates, crate_downloads, crate_versions, crate_default_versions, metadata, "Imported database dump");
    Ok(())
  }

  #[instrument(skip_all, err)]
  async fn is_import_required(&self) -> Result<bool, InternalError> {
    let conn = self.db_pool.get().await?;
    let import_count: i64 = conn.interact(|conn| {
      use att_core::schema::import_crates_metadata::dsl::*;
      import_crates_metadata.count().get_result(conn)
    }).await??;
    Ok(import_count == 0)
  }

  #[instrument(skip_all, err)]
  fn update_db_dump_file(&self) -> impl Future<Output=Result<bool, InternalError>> {
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
