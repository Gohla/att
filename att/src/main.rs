use std::error::Error;
use std::time::Duration;

use crates_io_api::AsyncClient;
use iced::{Application, Settings};

use crate::app::App;

pub mod app;
pub mod widget;
pub mod component;

fn main() -> Result<(), Box<dyn Error>> {
  let crates_io_api = AsyncClient::new("Gohla (https://github.com/Gohla)", Duration::from_secs(1))?;
  let flags = App::new(crates_io_api);
  let settings = Settings {
    antialiasing: true,
    ..Settings::with_flags(flags)
  };
  App::run(settings)?;
  Ok(())
}
