use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

use apalis::prelude::Monitor;
use tokio::signal;
use tokio::sync::RwLock;

use att_core::start::{DirectoryKind, Start};

use crate::app::{App, run};
use crate::krate::{Crates, RefreshCrates};

mod app;
mod data;
mod krate;

fn main() -> Result<(), Box<dyn Error>> {
  let start = Start::new("Server");

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let _runtime_guard = runtime.enter();

  let data = start.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();
  let data = Arc::new(RwLock::new(data));
  let data_local = data.clone();

  let crates = Crates::new("Gohla (https://github.com/Gohla)")?;

  let monitor = Monitor::new();
  let monitor = RefreshCrates::register_worker(monitor, crates.clone(), data.clone());
  runtime.spawn(monitor.run_with_signal(shutdown_signal_result()));

  let app = App::new(data, crates);
  runtime.block_on(run(app, shutdown_signal()))?;

  start.serialize_json_file(DirectoryKind::Data, "data.json", data_local.blocking_read().deref())?;

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

async fn shutdown_signal_result() -> Result<(), std::io::Error> {
  shutdown_signal().await;
  Ok(())
}
