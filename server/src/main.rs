use std::error::Error;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::RwLock;

use crate::app::{App, run};
use crate::data::Data;
use crate::krate::crates_io_client::CratesIoClient;

mod app;
mod async_util;
mod data;
mod krate;

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

  let project_dirs = directories::ProjectDirs::from("", "ATT", "Server");
  let data = if let Some(project_dirs) = &project_dirs {
    Data::deserialize_or_default(project_dirs)?
  } else {
    Data::default()
  };

  let data = Arc::new(RwLock::new(data));
  let data_local = data.clone();
  let crates_client = CratesIoClient::new("Gohla (https://github.com/Gohla)")?;
  let app = App::new(data, crates_client);

  runtime.block_on(run(app, shutdown_signal()))?;

  if let Some(project_dirs) = &project_dirs {
    data_local.blocking_read().serialize(&project_dirs)?;
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
