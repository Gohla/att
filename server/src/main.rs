use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

use att_core::start::{DirectoryKind, Start};

use crate::job_scheduler::{Job, JobOutput, JobScheduler};
use crate::krate::{Crates, CratesData, RefreshJob};
use crate::server::Server;

mod server;
mod krate;
mod job_scheduler;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub crates: CratesData,
}

fn main() -> Result<(), Box<dyn Error>> {
  let (start, _file_log_flush_guard) = Start::new("Server");
  let start = Arc::new(start);

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let _runtime_guard = runtime.enter();

  let data = start.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default();
  let data = Arc::new(RwLock::new(data));

  let crates = Crates::new("Gohla (https://github.com/Gohla)")?;

  let job_scheduler = JobScheduler::new();
  job_scheduler.schedule(interval(Duration::from_secs(60 * 60)), RefreshJob::new(crates.clone(), data.clone()));
  job_scheduler.schedule(interval(Duration::from_secs(60 * 5)), SerializeDataJob::new(start.clone(), data.clone()));

  let server = Server::new(data.clone(), crates.clone());
  runtime.block_on(server.run(shutdown_signal()))?;

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

pub struct SerializeDataJob {
  start: Arc<Start>,
  data: Arc<RwLock<Data>>,
}
impl SerializeDataJob {
  pub fn new(start: Arc<Start>, data: Arc<RwLock<Data>>) -> Self {
    Self { start, data }
  }
}
impl Job for SerializeDataJob {
  async fn run(&self) -> JobOutput {
    tracing::info!("running serialize data job");
    self.start.serialize_json_file(DirectoryKind::Data, "data.json", self.data.read().await.deref())?;
    Ok(())
  }
}
