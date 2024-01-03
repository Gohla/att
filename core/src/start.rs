use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufWriter};
use std::path::Path;

use directories::ProjectDirs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct Start {
  project_directories: Option<ProjectDirs>,
}
impl Start {
  pub fn new(application: &str) -> (Self, Option<WorkerGuard>) {
    #[cfg(target_arch = "wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let _ = dotenvy::dotenv(); // Ignore error ok: .env file is not required.

    let project_directories = ProjectDirs::from("", "ATT", application);

    let main_filter_layer = EnvFilter::from_env("MAIN_LOG");
    let layered = tracing_subscriber::registry();

    #[cfg(not(target_arch = "wasm32"))] let file_log_flush_guard = {
      let layered = layered.with(
        tracing_subscriber::fmt::layer()
          .with_writer(io::stderr)
          .with_filter(main_filter_layer)
      );
      let guard = if let Some(project_directories) = &project_directories {
        let log_dir = project_directories.data_local_dir();
        let log_file_path = log_dir.join("log.txt");
        create_dir_all(log_dir).unwrap();
        let log_file = File::create(log_file_path).unwrap();
        let writer = BufWriter::new(log_file);
        let (non_blocking, guard) = tracing_appender::non_blocking(writer);
        let layered = layered.with(
          tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_filter(EnvFilter::from_env("FILE_LOG"))
        );
        layered.init();
        Some(guard)
      } else {
        layered.init();
        None
      };
      guard
    };
    #[cfg(target_arch = "wasm32")] {
      layered
        .with(main_filter_layer)
        .with(tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default()))
        .init();
    }

    (Self { project_directories }, file_log_flush_guard)
  }
}

pub enum DirectoryKind {
  Data,
  LocalData,
  Cache,
}
impl Start {
  pub fn project_directories(&self) -> Option<&ProjectDirs> {
    self.project_directories.as_ref()
  }

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
}

#[cfg(feature = "start_serde_json")]
impl Start {
  pub fn deserialize_json_file<T: serde::de::DeserializeOwned>(
    &self,
    directory_kind: DirectoryKind,
    file_name: impl AsRef<Path>
  ) -> Result<Option<T>, io::Error> {
    let directory_path = self.directory(directory_kind);
    let file_path = directory_path.map(|p| p.join(file_name));

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
    let directory_path = self.directory(directory_kind);
    Self::create_directory_all_opt(directory_path)?;
    let file_path = directory_path.map(|p| p.join(file_name));

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
  fn create_directory_all_opt(directory_path: Option<impl AsRef<Path>>) -> Result<(), io::Error> {
    directory_path.map(|path| create_dir_all(path)).transpose()?;
    Ok(())
  }
}
