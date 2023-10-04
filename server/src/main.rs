use std::error::Error;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::RwLock;

use crate::app::{AppState, run};
use crate::crates_client::CratesClient;
use crate::data::{Cache, Data};

mod app;
mod crates_client;
mod async_util;
mod data;

fn main() -> Result<(), Box<dyn Error>> {
  let subscriber = tracing_subscriber::fmt()
    .finish();
  if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("Failed to set global tracing subscriber: {:?}", e);
  }

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let _runtime_guard = runtime.enter();

  let project_dirs = directories::ProjectDirs::from("", "", "ATT");
  let (data, cache) = if let Some(project_dirs) = &project_dirs {
    let data = Data::deserialize_or_default(project_dirs)?;
    let cache = Cache::deserialize_or_default(project_dirs)?;
    (data, cache)
  } else {
    let data = Data::default();
    let cache = Cache::default();
    (data, cache)
  };

  let data = Arc::new(RwLock::new(data));
  let cache = Arc::new(RwLock::new(cache));
  let crates_client = CratesClient::new("Gohla (https://github.com/Gohla)")?;
  let state = AppState { data, cache, crates_client };

  runtime.block_on(run(state.clone(), shutdown_signal()))?;

  if let Some(project_dirs) = &project_dirs {
    let data_result = state.data.blocking_read().serialize(&project_dirs);
    let cache_result = state.cache.blocking_read().serialize(&project_dirs);
    data_result?;
    cache_result?;
  }

  Ok(())
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  let terminate = {
    #[cfg(unix)] {
      async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
          .expect("failed to install signal handler")
          .recv()
          .await
      }
    }
    #[cfg(not(unix))] {
      std::future::pending::<()>()
    }
  };

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
