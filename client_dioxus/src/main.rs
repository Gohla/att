use std::error::Error;

use dioxus_web::Config;

use att_client::{AttClient, Data};
use att_core::app::env::{load_dotenv_into_env, run_or_compile_time_env};
use att_core::app::panic_handler::install_panic_handler;
use att_core::app::storage::{DirectoryKind, Storage};
use att_core::app::tracing::AppTracingBuilder;

use crate::app::{App, AppProps};

pub mod app;

fn main() -> Result<(), Box<dyn Error>> {
  install_panic_handler();
  load_dotenv_into_env();
  let storage = Storage::new("client_dioxus");
  let _tracing = AppTracingBuilder::default()
    .with_log_file_path_opt(storage.local_data_file("log.txt"))
    .build();

  let data: Data = storage.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();

  let base_url = run_or_compile_time_env!("ATT_CLIENT_BASE_URL");
  let client = AttClient::from_base_url(base_url)?;

  let app_props = AppProps::new(data, client);
  let config = Config::default()
    .rootname("ATT")
    .with_default_panic_hook(false);
  dioxus_web::launch_with_props(App, app_props, config);

  //storage.serialize_json_file(DirectoryKind::Data, "data.json", &data)?;

  Ok(())
}

