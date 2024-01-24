use std::fs::{create_dir_all, File};
use std::io::{self, BufWriter};
use std::path::Path;

use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Default)]
pub struct AppTracingBuilder<P> {
  log_file_path: Option<P>,
}
impl<P: AsRef<Path>> AppTracingBuilder<P> {
  pub fn with_log_file_path(mut self, log_file_path: P) -> Self {
    self.log_file_path = Some(log_file_path);
    self
  }
  pub fn with_log_file_path_opt(mut self, log_file_path: Option<P>) -> Self {
    self.log_file_path = log_file_path;
    self
  }

  pub fn build(self) -> AppTracing {
    AppTracing::new(self.log_file_path.as_ref())
  }
}

pub struct AppTracing {
  _file_log_flush_guard: Option<WorkerGuard>,
}
impl AppTracing {
  fn new(log_file_path: Option<impl AsRef<Path>>) -> Self {
    #[cfg(target_arch = "wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    macro_rules! filter {
      ($env:literal) => {{
        #[cfg(feature = "app_env")] {
          EnvFilter::new(crate::app::env::run_or_compile_time_env!($env))
        }
        #[cfg(not(feature = "app_env"))] {
          EnvFilter::try_from_env($env).unwrap_or_default()
        }
      }};
    }

    let console_filter = filter!("CONSOLE_LOG");

    #[cfg(not(target_arch = "wasm32"))] let _file_log_flush_guard = {
      let layered = tracing_subscriber::registry();
      let layered = layered.with(
        tracing_subscriber::fmt::layer()
          .with_writer(io::stderr)
          .with_filter(console_filter)
      );
      let guard = if let Some(log_file_path) = &log_file_path {
        let log_file_path = log_file_path.as_ref();
        let result = (|| {
          if let Some(parent) = log_file_path.parent() {
            create_dir_all(parent)?;
          }
          File::create(log_file_path)
        })();
        match result {
          Err(e) => {
            error!("Cannot log to file; could not truncate/create and open log file '{}' for writing: {}", log_file_path.display(), e);
            None
          }
          Ok(log_file) => {
            let writer = BufWriter::new(log_file);
            let (non_blocking, guard) = tracing_appender::non_blocking(writer);
            let filter = filter!("FILE_LOG");
            let layered = layered.with(
              tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(filter)
            );
            layered.init();
            Some(guard)
          }
        }
      } else {
        layered.init();
        None
      };
      guard
    };

    #[cfg(target_arch = "wasm32")] let _file_log_flush_guard = {
      let layered = tracing_subscriber::registry();
      let layered = layered.with(
        tracing_subscriber::fmt::layer()
          .with_writer(tracing_web::MakeWebConsoleWriter::new())
          .with_ansi(false)
          .without_time()
          .with_filter(console_filter)
      );
      layered.init();
      None
    };

    Self { _file_log_flush_guard }
  }
}
