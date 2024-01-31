use std::borrow::Cow;
use std::error::Error;

use iced::{Application, Settings, window};
use iced::window::settings::PlatformSpecific;

use att_client::http_client::AttHttpClient;
use att_core::app::env;
use att_core::app::storage::{DirectoryKind, Storage};
use att_core::app::tracing::AppTracingBuilder;
use att_core::run_or_compile_time_env;

use crate::app::{App, Flags};
use crate::widget::icon;

pub mod widget;
pub mod update;
pub mod app;

fn main() -> Result<(), Box<dyn Error>> {
  env::load_dotenv_into_env();
  let storage = Storage::new("client_iced");
  let _tracing = AppTracingBuilder::default()
    .with_log_file_path_opt(storage.local_data_file("log.txt"))
    .build();

  let data = storage.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();
  let save_fn = Box::new(move |data: &_| {
    storage.serialize_json_file(DirectoryKind::Data, "data.json", data)?;
    Ok(())
  });

  let base_url = run_or_compile_time_env!("ATT_CLIENT_BASE_URL");
  let http_client = AttHttpClient::from_base_url(base_url)?;

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
    http_client,
    save_fn,
    data,
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
