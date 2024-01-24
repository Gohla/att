use std::error::Error;

use att_client::Data;
use att_client::http_client::AttHttpClient;
use att_core::app::env;
use att_core::app::env::run_or_compile_time_env;
use att_core::app::storage::{DirectoryKind, Storage};
use att_core::app::tracing::AppTracingBuilder;

use crate::app::app;

pub mod app;

fn main() -> Result<(), Box<dyn Error>> {
  env::load_dotenv_into_env();
  let storage = Storage::new("client_dioxus");
  let _tracing = AppTracingBuilder::default()
    .with_log_file_path_opt(storage.local_data_file("log.txt"))
    .build();

  let data: Data = storage.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();

  let base_url = run_or_compile_time_env!("ATT_CLIENT_BASE_URL");
  let _client = AttHttpClient::from_base_url(base_url)?;

  dioxus_web::launch(app);

  storage.serialize_json_file(DirectoryKind::Data, "data.json", &data)?;

  Ok(())
}

