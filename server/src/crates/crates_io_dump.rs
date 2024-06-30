use std::error::Error;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use db_dump::{crate_downloads, crates, default_versions, Loader, versions};
use futures::StreamExt;
use nohash::IntMap;
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

  crate_id_to_crate: IntMap<u32, crates::Row>,
  crate_id_to_downloads: IntMap<u32, crate_downloads::Row>,
  crate_id_to_default_version: IntMap<u32, default_versions::Row>,
  version_id_to_version: IntMap<u32, versions::Row>,

  crate_name_trie: Trie<u8, u32>,
}

impl CratesIoDump {
  #[inline]
  pub fn new(db_dump_file: PathBuf) -> Self {
    Self {
      db_dump_file,
      is_loaded: false,

      crate_id_to_crate: Default::default(),
      crate_id_to_downloads: Default::default(),
      crate_id_to_default_version: Default::default(),
      version_id_to_version: Default::default(),

      crate_name_trie: TrieBuilder::new().build()
    }
  }

  pub fn search(&self, search_term: String) -> impl Iterator<Item=Crate> + '_ {
    self.crate_name_trie.postfix_search::<String, _>(&search_term)
      .flat_map(|(_, crate_id)| self.crate_id_to_crate.get(crate_id).map(|row|{
        let downloads = self.crate_id_to_downloads.get(crate_id).map(|row|row.downloads).unwrap_or_default();
        let max_version = self.crate_id_to_default_version.get(crate_id)
          .and_then(|row|self.version_id_to_version.get(&row.version_id.0))
          .map(|row|row.num.to_string())
          .unwrap_or_default();
        Crate {
          id: row.name.clone(),
          downloads,
          updated_at: row.updated_at,
          max_version,
        }
      }))
  }
}

pub const UPDATE_DURATION: Duration = Duration::from_secs(60 * 60 * 24);

pub const DB_DUMP_URL: &str = "https://static.crates.io/db-dump.tar.gz";


// Internals

impl CratesIoDump {
  #[instrument(skip_all, err)]
  fn update_crates(&mut self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    info!("Updating crates trie from database dump");

    self.crate_id_to_crate.clear();
    self.crate_id_to_downloads.clear();
    self.crate_id_to_default_version.clear();
    self.version_id_to_version.clear();

    let mut trie_builder = TrieBuilder::new();
    Loader::new()
      .crates(|row| {
        let id = row.id.0;
        trie_builder.insert(row.name.bytes(), id);
        self.crate_id_to_crate.insert(id, row);
      })
      .crate_downloads(|row| {
        self.crate_id_to_downloads.insert(row.crate_id.0, row);
      })
      .default_versions(|row| {
        self.crate_id_to_default_version.insert(row.crate_id.0, row);
      })
      .versions(|row| {
        self.version_id_to_version.insert(row.id.0, row);
      })
      .load(&self.db_dump_file)?;

    self.crate_name_trie = trie_builder.build();
    self.is_loaded = true;

    info!("Updated crates trie: {} entries", self.crate_id_to_crate.len());

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
