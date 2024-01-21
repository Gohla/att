use std::borrow::Cow;
use std::error::Error;

use iced::{Application, Settings, window};
use iced::window::settings::PlatformSpecific;

use att_client::AttHttpClient;
use att_core::util::dotenv;
use att_core::util::start::{DirectoryKind, Start};

use crate::app::{App, Flags};
use crate::widget::icon;

pub mod app;
pub mod widget;
pub mod component;

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

  let platform_specific: PlatformSpecific;
  #[cfg(not(target_arch = "wasm32"))] {
    platform_specific = Default::default();
  }
  #[cfg(target_arch = "wasm32")]{
    platform_specific = PlatformSpecific { target: Some("canvas".to_string()), ..Default::default() };
  }
  let window = window::Settings {
    platform_specific,
    exit_on_close_request: false,
    ..Default::default()
  };

  let fonts = vec![
    Cow::Borrowed(icon::FONT_BYTES),
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
    id: Some("att".to_string()),
    window,
    fonts,
    ..Settings::with_flags(flags)
  };
  App::run(settings)?;

  Ok(())
}
