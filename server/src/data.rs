use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use crates_io_api::Crate;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub blessed_crate_ids: BTreeSet<String>,
}
impl Data {
  pub fn deserialize_or_default(project_dirs: &ProjectDirs) -> Result<Self, Box<dyn Error>> {
    let (_, file_path) = Self::paths(project_dirs);
    let data = from_json_file(file_path)?.unwrap_or_default();
    Ok(data)
  }

  pub fn serialize(&self, project_dirs: &ProjectDirs) -> Result<(), Box<dyn Error>> {
    let (directory_path, file_path) = Self::paths(project_dirs);
    create_dir_all(directory_path)?;
    to_json_file(file_path, self)?;
    Ok(())
  }

  fn paths(project_dirs: &ProjectDirs) -> (&Path, PathBuf) {
    let directory_path = project_dirs.data_dir();
    let file_path = directory_path.join("data.json");
    (directory_path, file_path)
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Cache {
  pub crate_data: BTreeMap<String, (Crate, DateTime<Utc>)>
}
impl Cache {
  pub fn deserialize_or_default(project_dirs: &ProjectDirs) -> Result<Self, Box<dyn Error>> {
    let (_, file_path) = Self::paths(project_dirs);
    let cache = from_json_file(file_path)?.unwrap_or_default();
    Ok(cache)
  }

  pub fn serialize(&self, project_dirs: &ProjectDirs) -> Result<(), Box<dyn Error>> {
    let (directory_path, file_path) = Self::paths(project_dirs);
    create_dir_all(directory_path)?;
    to_json_file(file_path, self)?;
    Ok(())
  }

  fn paths(project_dirs: &ProjectDirs) -> (&Path, PathBuf) {
    let directory_path = project_dirs.cache_dir();
    let file_path = directory_path.join("cache.json");
    (directory_path, file_path)
  }
}

fn from_json_file<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<Option<T>, Box<dyn Error>> {
  let mut open_options = OpenOptions::new();
  open_options.read(true);
  let file_opt = open_file(path, open_options)?;
  let result = file_opt.map(|file| serde_json::from_reader(io::BufReader::new(file))).transpose();
  if let Err(cause) = &result {
    if cause.classify() == serde_json::error::Category::Data {
      tracing::error!(%cause, "failed to deserialize JSON due to data format changes; returning None");
      return Ok(None)
    }
  }
  Ok(result?)
}
fn to_json_file<T: serde::Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), Box<dyn Error>> {
  let mut open_options = OpenOptions::new();
  open_options.write(true).truncate(true).create(true);
  let file_opt = open_file(path, open_options)?;
  file_opt.map(|file| serde_json::to_writer(io::BufWriter::new(file), value)).transpose()?;
  Ok(())
}
fn open_file(path: impl AsRef<Path>, open_options: OpenOptions) -> Result<Option<File>, io::Error> {
  let file = match open_options.open(path) {
    Err(e) if e.kind() == io::ErrorKind::NotFound => None,
    Err(e) => return Err(e),
    Ok(file) => Some(file),
  };
  Ok(file)
}
