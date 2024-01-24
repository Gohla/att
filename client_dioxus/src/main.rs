use std::error::Error;

use att_client::http_client::AttHttpClient;
use att_core::app::env::run_or_compile_time_env;
use att_core::app::storage::Storage;

use crate::app::app;

pub mod app;

fn main() -> Result<(), Box<dyn Error>> {
  let _storage = Storage::new("client_dioxus");
  // let data = start.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();
  // let cache = start.deserialize_json_file(DirectoryKind::Cache, "cache.json")?.unwrap_or_default();
  // let save_fn = Box::new(move |data: &_, cache: &_| {
  //   start.serialize_json_file(DirectoryKind::Data, "data.json", data)?;
  //   start.serialize_json_file(DirectoryKind::Cache, "cache.json", cache)?;
  //   Ok(())
  // });

  let base_url = run_or_compile_time_env!("ATT_CLIENT_BASE_URL");
  let _client = AttHttpClient::from_base_url(base_url)?;

  dioxus_web::launch(app);

  // start.serialize_json_file(DirectoryKind::Data, "data.json", data)?;
  // start.serialize_json_file(DirectoryKind::Cache, "cache.json", cache)?;

  Ok(())
}

