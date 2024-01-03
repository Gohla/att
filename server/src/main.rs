use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

use att_core::start::{DirectoryKind, Start};

use crate::app::{App, run};
use crate::job_scheduler::JobScheduler;
use crate::krate::{Crates, RefreshJob};

mod app;
mod data;
mod krate;
mod job_scheduler;

fn main() -> Result<(), Box<dyn Error>> {
  let start = Start::new("Server");

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let _runtime_guard = runtime.enter();

  let data = start.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();
  let data = Arc::new(RwLock::new(data));

  let crates = Crates::new("Gohla (https://github.com/Gohla)")?;

  let job_scheduler = JobScheduler::new();
  job_scheduler.schedule(interval(Duration::from_secs(60 * 60)), RefreshJob::new(crates.clone(), data.clone()));

  let app = App::new(data.clone(), crates);
  runtime.block_on(run(app, shutdown_signal()))?;

  start.serialize_json_file(DirectoryKind::Data, "data.json", data.blocking_read().deref())?;

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
