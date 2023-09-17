use std::error::Error;
use std::time::Duration;

use crates_io_api::AsyncClient;
use iced::{Application, Settings};

use crate::app::App;

pub mod app;
pub mod util;
pub mod add_crate;
pub mod modal;

fn main() -> Result<(), Box<dyn Error>> {
  let crates_io_api = AsyncClient::new("Gohla (https://github.com/Gohla)", Duration::from_secs(1))?;
  let app = App::new(crates_io_api);
  App::run(Settings::with_flags(app))?;
  Ok(())
}
