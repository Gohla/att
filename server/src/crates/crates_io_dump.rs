use std::error::Error;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use db_dump::Loader;
use futures::StreamExt;
use tokio::fs;
use tokio::fs::File;
use tokio::task::block_in_place;
use tracing::{info, instrument};
use trie_rs::map::{Trie, TrieBuilder};

use att_core::crates::Crate;

use crate::job_scheduler::{Job, JobAction, JobResult};

pub struct CratesIoDump {
  db_dump_file: PathBuf,
  is_loaded: bool,
  crates: Trie<u8, Crate>,
}

impl CratesIoDump {
  #[inline]
  pub fn new(db_dump_file: PathBuf) -> Self {
    Self { db_dump_file, is_loaded: false, crates: TrieBuilder::new().build() }
  }

  #[inline]
  pub fn crates(&self) -> &Trie<u8, Crate> {
    &self.crates
  }
}

pub const UPDATE_DURATION: Duration = Duration::from_secs(60 * 60 * 24);

pub const DB_DUMP_URL: &str = "https://static.crates.io/db-dump.tar.gz";


// Internals

impl CratesIoDump {
  #[instrument(skip_all, err)]
  fn update_crates(&mut self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    info!("Updating crates trie from database dump");

    let mut builder = TrieBuilder::new();
    let mut len = 0usize;
    Loader::new()
      .crates(|row| {
        let krate = Crate {
          id: row.name.clone(),
          downloads: 0,
          updated_at: row.updated_at,
          max_version: String::new(),
        };
        builder.push(row.name, krate);
        len += 1;
      })
      .load(&self.db_dump_file)?;

    self.crates = builder.build();
    self.is_loaded = true;

    info!("Updated crates trie: {} entries", len);

    Ok(())
  }

  #[instrument(skip_all, err)]
  fn update_db_dump_file(&self) -> impl Future<Output=Result<bool, Box<dyn Error + Send + Sync + 'static>>> {
    let db_dump_file = self.db_dump_file.clone();

    async move {
      let is_up_to_date = match metadata(&db_dump_file).await? {
        None => false,
        Some(metadata) => metadata.modified()?.elapsed()? < UPDATE_DURATION,
      };
      if is_up_to_date {
        return Ok(false)
      }

      info!("Downloading crates.io database dump '{}' into '{}'", DB_DUMP_URL, db_dump_file.display());

      if let Some(parent) = db_dump_file.parent() {
        fs::create_dir_all(parent).await?;
      }
      let mut file = File::create(&db_dump_file).await?;

      let response = reqwest::get(DB_DUMP_URL).await?;
      let mut bytes_stream = response.bytes_stream();

      while let Some(bytes) = bytes_stream.next().await {
        let bytes = bytes?;
        tokio::io::copy(&mut bytes.as_ref(), &mut file).await?;
      }
      Ok(true)
    }
  }
}


/// Gets the metadata for given `path`, returning:
///
/// - `Ok(Some(metadata))` if a file or directory exists at given path,
/// - `Ok(None)` if no file or directory exists at given path,
/// - `Err(e)` if there was an error getting the metadata for given path.
#[inline]
async fn metadata(path: impl AsRef<Path>) -> Result<Option<Metadata>, io::Error> {
  match fs::metadata(path).await {
    Ok(m) => Ok(Some(m)),
    Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
    Err(e) => Err(e),
  }
}


// Scheduled job

pub struct UpdateCratesIoDumpJob {
  crates_io_dump: Arc<RwLock<CratesIoDump>>
}

impl UpdateCratesIoDumpJob {
  pub fn new(crates_io_dump: Arc<RwLock<CratesIoDump>>) -> Self {
    Self { crates_io_dump }
  }
}

impl Job for UpdateCratesIoDumpJob {
  async fn run(&self) -> JobResult {
    let (update_future, is_loaded) = {
      let read = self.crates_io_dump.read().unwrap();
      let future = read.update_db_dump_file();
      (future, read.is_loaded)
    };
    let file_updated = update_future.await?;

    block_in_place(|| {
      if file_updated || !is_loaded {
        self.crates_io_dump.write().unwrap().update_crates()?;
      }
      Ok::<(), Box<dyn Error + Send + Sync + 'static>>(())
    })?;

    Ok(JobAction::Continue)
  }
}
