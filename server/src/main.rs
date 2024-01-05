use std::error::Error;

use tokio::runtime::Runtime;
use tokio::signal;
use tokio::time::{Duration, interval};
use tracing::debug;

use att_core::start::Start;

use crate::data::{Database, StoreDatabaseJob};
use crate::job_scheduler::JobScheduler;
use crate::krate::{Crates, RefreshJob};
use crate::server::Server;

mod server;
mod krate;
mod job_scheduler;
mod data;

fn main() -> Result<(), Box<dyn Error>> {
  let (start, _file_log_flush_guard) = Start::new("Server");

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let runtime_guard = runtime.enter();

  let result = run(start, &runtime);

  debug!("shutting down tokio runtime..");
  drop(runtime_guard);
  runtime.shutdown_timeout(Duration::from_secs(10));
  debug!("..done shutting down tokio runtime");

  result
}

fn run(start: Start, runtime: &Runtime) -> Result<(), Box<dyn Error>> {
  let database = Database::blocking_deserialize(&start)?;

  let (crates, crates_io_client_task) = Crates::new("Gohla (https://github.com/Gohla)")?;
  runtime.spawn(crates_io_client_task);

  let (job_scheduler, job_scheduler_task) = JobScheduler::new();
  runtime.spawn(job_scheduler_task);
  job_scheduler.blocking_schedule_job(RefreshJob::new(crates.clone(), database.clone()), interval(Duration::from_secs(60 * 60)), "refresh outdated crate data");
  job_scheduler.blocking_schedule_blocking_job(StoreDatabaseJob::new(start.clone(), database.clone()), interval(Duration::from_secs(60 * 5)), "store database");

  let server = Server::new(database.clone(), crates.clone());
  let result = runtime.block_on(server.run(shutdown_signal()));

  debug!("storing database");
  database.blocking_serialize(&start)?;

  result
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
