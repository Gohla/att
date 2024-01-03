use std::error::Error;

use tokio::signal;
use tokio::time::{Duration, interval};

use att_core::start::Start;

use crate::data::{Database, SerializeDataJob};
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
  let _runtime_guard = runtime.enter();

  let database = Database::blocking_deserialize(&start)?;

  let crates = Crates::new("Gohla (https://github.com/Gohla)")?;

  let job_scheduler = JobScheduler::new();
  job_scheduler.blocking_schedule(interval(Duration::from_secs(60 * 60)), RefreshJob::new(crates.clone(), database.clone()));
  job_scheduler.blocking_schedule(interval(Duration::from_secs(60 * 5)), SerializeDataJob::new(start.clone(), database.clone()));

  let server = Server::new(database.clone(), crates.clone());
  runtime.block_on(server.run(shutdown_signal()))?;

  database.blocking_serialize(&start)?;

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
