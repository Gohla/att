use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

#[derive(Default, Clone, Debug)]
pub struct Storage {
  project_directories: Option<ProjectDirs>,
}
impl Storage {
  pub fn new(application: &str) -> Self {
    let project_directories = ProjectDirs::from("", "ATT", application);
    Self { project_directories }
  }

  pub fn project_directories(&self) -> Option<&ProjectDirs> {
    self.project_directories.as_ref()
  }
}

pub enum DirectoryKind {
  Data,
  LocalData,
  Cache,
}
impl Storage {
  pub fn directory(&self, kind: DirectoryKind) -> Option<&Path> {
    if let Some(project_directories) = &self.project_directories {
      let path = match kind {
        DirectoryKind::Data => project_directories.data_dir(),
        DirectoryKind::LocalData => project_directories.data_local_dir(),
        DirectoryKind::Cache => project_directories.cache_dir(),
      };
      Some(path)
    } else {
      None
    }
  }
  pub fn data_directory(&self) -> Option<&Path> {
    self.directory(DirectoryKind::Data)
  }
  pub fn local_data_directory(&self) -> Option<&Path> {
    self.directory(DirectoryKind::LocalData)
  }
  pub fn cache_directory(&self) -> Option<&Path> {
    self.directory(DirectoryKind::Cache)
  }

  pub fn file(&self, kind: DirectoryKind, file_path: impl AsRef<Path>) -> Option<PathBuf> {
    self.directory(kind).map(|d| d.join(file_path))
  }
  pub fn data_file(&self, file_path: impl AsRef<Path>) -> Option<PathBuf> {
    self.file(DirectoryKind::Data, file_path)
  }
  pub fn local_data_file(&self, file_path: impl AsRef<Path>) -> Option<PathBuf> {
    self.file(DirectoryKind::LocalData, file_path)
  }
  pub fn cache_file(&self, file_path: impl AsRef<Path>) -> Option<PathBuf> {
    self.file(DirectoryKind::Cache, file_path)
  }
}

#[cfg(feature = "app_storage_json")]
impl Storage {
  pub fn deserialize_json_file<T: serde::de::DeserializeOwned>(
    &self,
    directory_kind: DirectoryKind,
    file_name: impl AsRef<Path>
  ) -> Result<Option<T>, io::Error> {
    let file_path = self.file(directory_kind, file_name);

    let mut open_options = OpenOptions::new();
    open_options.read(true);
    let file_opt = Self::open_file_opt(file_path, open_options)?;
    let result = file_opt.map(|file| serde_json::from_reader(io::BufReader::new(file))).transpose();
    if let Err(cause) = &result {
      if cause.classify() == serde_json::error::Category::Data {
        tracing::error!(%cause, "failed to deserialize JSON due to data format changes; returning None");
        return Ok(None)
      }
    }
    Ok(result?)
  }
  pub fn serialize_json_file<T: serde::Serialize>(
    &self,
    directory_kind: DirectoryKind,
    file_name: impl AsRef<Path>,
    value: &T
  ) -> Result<(), io::Error> {
    let file_path = self.file(directory_kind, file_name);
    if let Some(parent) = file_path.as_ref().and_then(|p| p.parent()) {
      create_dir_all(parent)?;
    }

    let mut open_options = OpenOptions::new();
    open_options.write(true).truncate(true).create(true);
    let file_opt = Self::open_file_opt(file_path, open_options)?;
    file_opt.map(|file| serde_json::to_writer(BufWriter::new(file), value)).transpose()?;
    Ok(())
  }

  fn open_file_opt(file_path: Option<impl AsRef<Path>>, open_options: OpenOptions) -> Result<Option<File>, io::Error> {
    file_path.and_then(|path| match open_options.open(path) {
      Err(e) if e.kind() == io::ErrorKind::NotFound => None,
      v => Some(v),
    }).transpose()
  }
}
