use std::error::Error;

use tokio::runtime::Runtime;
use tokio::signal;
use tokio::time::{Duration, interval};
use tracing::debug;

use att_core::app::env;
use att_core::app::storage::Storage;
use att_core::app::tracing::AppTracingBuilder;
use att_server_db::DbPool;

use crate::crates::{Crates, crates_io_dump};
use crate::job_scheduler::JobScheduler;
use crate::server::Server;
use crate::users::Users;

pub mod server;
pub mod crates;
pub mod job_scheduler;
pub mod users;
pub mod util;

fn main() -> Result<(), Box<dyn Error>> {
  env::load_dotenv_into_env();
  let storage = Storage::new("server");
  let _tracing = AppTracingBuilder::default()
    .with_log_file_path_opt(storage.local_data_file("log.txt"))
    .build();

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  let runtime_guard = runtime.enter();

  let db_pool = DbPool::new()?;

  let result = run(storage, &runtime, db_pool);

  debug!("shutting down tokio runtime..");
  drop(runtime_guard);
  runtime.shutdown_timeout(Duration::from_secs(10));
  debug!("..done shutting down tokio runtime");

  result
}

fn run(storage: Storage, runtime: &Runtime, db_pool: DbPool) -> Result<(), Box<dyn Error>> {
  let users = Users::from_db_pool(db_pool.clone());

  let (crates, crates_io_client_task) = Crates::new(
    db_pool,
    "Gohla (https://github.com/Gohla)",
    storage.cache_file("db-dump.tar.gz").unwrap(),
  )?;
  runtime.spawn(crates_io_client_task);

  let (job_scheduler, job_scheduler_task) = JobScheduler::new();
  runtime.spawn(job_scheduler_task);
  job_scheduler.blocking_schedule_job(crates.create_update_crates_io_dump_job(), interval(crates_io_dump::UPDATE_DURATION), "update crates.io database dump");

  let server = Server::new(users, crates);
  let result = runtime.block_on(server.run(shutdown_signal()));

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
