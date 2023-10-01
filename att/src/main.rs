use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::path::Path;
use std::time::Duration;

use crates_io_api::AsyncClient;
use iced::{Application, Settings};

use crate::app::{App, Flags};

pub mod app;
pub mod widget;
pub mod component;
pub mod crates_client;

fn main() -> Result<(), Box<dyn Error>> {
  let subscriber = tracing_subscriber::fmt()
    .finish();
  if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("Failed to set global tracing subscriber: {:?}", e);
  }

  let directories = directories::ProjectDirs::from("", "", "ATT");
  let data_directory_path = directories.as_ref().map(|d| d.data_dir().to_path_buf());
  let data_file_path = data_directory_path.as_ref().map(|p| p.join("data.json"));
  let cache_directory_path = directories.as_ref().map(|d| d.cache_dir().to_path_buf());
  let cache_file_path = cache_directory_path.as_ref().map(|p| p.join("cache.json"));

  let model = from_json_file_opt(data_file_path.as_ref())?;
  let cache = from_json_file_opt(cache_file_path.as_ref())?;

  let dark_mode = match dark_light::detect() {
    dark_light::Mode::Dark => true,
    dark_light::Mode::Light | dark_light::Mode::Default => false,
  };

  let save_fn = Box::new(move |model: &_, cache: &_| {
    create_dir_all_opt(data_directory_path.clone())?;
    to_json_file_opt(data_file_path.clone(), model)?;
    create_dir_all_opt(cache_directory_path.clone())?;
    to_json_file_opt(cache_file_path.clone(), cache)?;
    Ok(())
  });

  let crates_io_api = AsyncClient::new("Gohla (https://github.com/Gohla)", Duration::from_secs(1))?;

  let flags = Flags {
    model,
    cache,
    dark_mode,
    save_fn,
    crates_io_api,
  };
  let settings = Settings {
    exit_on_close_request: false,
    ..Settings::with_flags(flags)
  };
  App::run(settings)?;

  Ok(())
}

fn from_json_file_opt<T: serde::de::DeserializeOwned>(path: Option<impl AsRef<Path>>) -> Result<Option<T>, Box<dyn Error>> {
  let mut open_options = OpenOptions::new();
  open_options.read(true);
  let file_opt = open_file_opt(path, open_options)?;
  let value_opt = file_opt.map(|file| serde_json::from_reader(io::BufReader::new(file))).transpose()?;
  Ok(value_opt)
}
fn to_json_file_opt<T: serde::Serialize>(path: Option<impl AsRef<Path>>, value: &T) -> Result<(), Box<dyn Error>> {
  let mut open_options = OpenOptions::new();
  open_options.write(true).truncate(true).create(true);
  let file_opt = open_file_opt(path, open_options)?;
  file_opt.map(|file| serde_json::to_writer(io::BufWriter::new(file), value)).transpose()?;
  Ok(())
}
fn open_file_opt(path: Option<impl AsRef<Path>>, open_options: OpenOptions) -> Result<Option<File>, io::Error> {
  path.and_then(|path| match open_options.open(path) {
    Err(e) if e.kind() == io::ErrorKind::NotFound => None,
    v => Some(v),
  }).transpose()
}
fn create_dir_all_opt(path: Option<impl AsRef<Path>>) -> Result<(), io::Error> {
  path.map(|path| create_dir_all(path)).transpose()?;
  Ok(())
}
