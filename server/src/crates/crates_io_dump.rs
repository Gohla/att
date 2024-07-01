use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTimeError};

use chrono::Utc;
use db_dump::Loader;
use diesel::prelude::*;
use futures::StreamExt;
use thiserror::Error;
use tokio::fs;
use tokio::fs::File;
use tokio::task::block_in_place;
use tracing::{debug, info, instrument};

use att_core::crates::{Crate, CrateDefaultVersion, CrateDownloads, CrateVersion};

use crate::db::{DbError, DbPool};
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


// Scheduled job

pub const UPDATE_DURATION: Duration = Duration::from_secs(60 * 60 * 24);

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


// Internals

#[derive(Debug, Error)]
enum InternalError {
  #[error(transparent)]
  DbDump(#[from] db_dump::Error),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Time(#[from] SystemTimeError),
  #[error(transparent)]
  HttpRequest(#[from] reqwest::Error),
  #[error(transparent)]
  Database(DbError),
}
impl<E: Into<DbError>> From<E> for InternalError {
  fn from(value: E) -> Self {
    Self::Database(value.into())
  }
}

impl CratesIoDump {
  #[instrument(skip_all, err)]
  async fn import_db_dump(&self) -> Result<(), InternalError> {
    info!("Reading database dump");

    let mut krates = Vec::new();
    let mut downloads = Vec::new();
    let mut versions = Vec::new();
    let mut default_versions = Vec::new();

    block_in_place(|| Loader::new()
      .crates(|row| {
        krates.push(Crate {
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
        downloads.push(CrateDownloads {
          crate_id: row.crate_id.0 as i32,
          downloads: row.downloads as i64,
        });
      })
      .versions(|row| {
        versions.push(CrateVersion {
          id: row.id.0 as i32,
          crate_id: row.crate_id.0 as i32,
          number: row.num.to_string(),
        });
      })
      .default_versions(|row| {
        default_versions.push(CrateDefaultVersion {
          crate_id: row.crate_id.0 as i32,
          version_id: row.version_id.0 as i32,
        });
      })
      .load(&self.db_dump_file)
    )?;

    info!("Importing database dump");

    let conn = self.db_pool.get().await?;
    let inserted_rows = conn.interact(move |conn| {
      conn.transaction(|conn| {
        use att_core::schema::{crate_default_versions, crate_downloads, crate_versions, crates, import_crates_metadata};
        use diesel::{copy_from, delete, insert_into};

        let mut inserted_rows: usize = 0;

        debug!("Deleting table `crate_default_versions`");
        delete(crate_default_versions::table).execute(conn)?;
        debug!("Deleting table `crate_versions`");
        delete(crate_versions::table).execute(conn)?;
        debug!("Deleting table `crate_downloads`");
        delete(crate_downloads::table).execute(conn)?;
        debug!("Deleting table `crates`");
        delete(crates::table).execute(conn)?;

        debug!("Copying {} crates into `crates`", krates.len());
        inserted_rows += copy_from(crates::table)
          .from_insertable(&krates)
          .execute(conn)?;

        debug!("Copying {} downloads into `crate_downloads`", downloads.len());
        inserted_rows += copy_from(crate_downloads::table)
          .from_insertable(&downloads)
          .execute(conn)?;

        debug!("Copying {} versions into `crate_versions`", versions.len());
        inserted_rows += copy_from(crate_versions::table)
          .from_insertable(&versions)
          .execute(conn)?;

        debug!("Copying {} default versions into `crate_default_versions`", default_versions.len());
        inserted_rows += copy_from(crate_default_versions::table)
          .from_insertable(&default_versions)
          .execute(conn)?;

        debug!("Inserting entry into `import_crates_metadata`");
        inserted_rows += insert_into(import_crates_metadata::table)
          .values(import_crates_metadata::imported_at.eq(Utc::now()))
          .execute(conn)?;

        Ok::<_, InternalError>(inserted_rows)
      })
    }).await??;

    info!(inserted_rows, "Imported database dump");
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
