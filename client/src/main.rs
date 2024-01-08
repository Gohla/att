use std::borrow::Cow;
use std::error::Error;

use iced::{Application, Settings, window};

use att_core::dotenv;
use att_core::start::{DirectoryKind, Start};

use crate::app::{App, Flags};
use crate::client::AttHttpClient;
use crate::widget::font::ICON_FONT_BYTES;

pub mod app;
pub mod widget;
pub mod component;
pub mod client;
mod time;

fn main() -> Result<(), Box<dyn Error>> {
  let (start, _file_log_flush_guard) = Start::new("Client");
  let data = start.deserialize_json_file(DirectoryKind::Data, "data.json")?;
  let cache = start.deserialize_json_file(DirectoryKind::Cache, "cache.json")?;
  let save_fn = Box::new(move |data: &_, cache: &_| {
    start.serialize_json_file(DirectoryKind::Data, "data.json", data)?;
    start.serialize_json_file(DirectoryKind::Cache, "cache.json", cache)?;
    Ok(())
  });

  let base_url = std::env::var("ATT_CLIENT_BASE_URL").unwrap_or_else(|_| dotenv!("ATT_CLIENT_BASE_URL").to_string());
  let client = AttHttpClient::from_base_url(base_url)?;

  let dark_mode = match dark_light::detect() {
    dark_light::Mode::Dark => true,
    dark_light::Mode::Light | dark_light::Mode::Default => false,
  };

  let id = Some("att".to_string());
  let window = window::Settings {
    exit_on_close_request: false,
    ..Default::default()
  };
  let fonts = vec![
    Cow::Borrowed(ICON_FONT_BYTES),
    #[cfg(target_arch = "wasm32")] Cow::Borrowed(widget::font::FIRA_SANS_FONT_BYTES)
  ];

  let flags = Flags {
    data,
    cache,
    save_fn,
    client,
    dark_mode,
  };
  let settings = Settings {
    id,
    window,
    fonts,
    ..Settings::with_flags(flags)
  };
  App::run(settings)?;

  Ok(())
}
