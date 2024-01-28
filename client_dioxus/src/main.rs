use std::error::Error;

use dioxus_web::Config;

use att_client::AttClient;
use att_core::app::env::{load_dotenv_into_env, run_or_compile_time_env};
use att_core::app::panic_handler::install_panic_handler;
use att_core::app::tracing::AppTracingBuilder;

use crate::app::{App, AppProps};

pub mod hook;
pub mod app;
pub mod component;

fn main() -> Result<(), Box<dyn Error>> {
  install_panic_handler();
  load_dotenv_into_env();
  let _tracing = AppTracingBuilder::default().build();

  let base_url = run_or_compile_time_env!("ATT_CLIENT_BASE_URL");
  let client = AttClient::from_base_url(base_url)?;

  let app_props = AppProps::new(client);
  let config = Config::default()
    .with_default_panic_hook(false);
  dioxus_web::launch_with_props(App, app_props, config);

  Ok(())
}
