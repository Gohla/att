use std::fs::{create_dir_all, File};
use std::io::{self, BufWriter};
use std::path::Path;

use tracing::warn;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Default)]
pub struct AppTracingBuilder<P> {
  console_filter: Option<EnvFilter>,
  log_file_path: Option<P>,
  file_filter: Option<EnvFilter>,
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

    let console_filter = self.console_filter.unwrap_or_else(|| filter!("CONSOLE_LOG"));
    let file = self.log_file_path.as_ref().map(|p| (p.as_ref(), self.file_filter.unwrap_or_else(|| filter!("FILE_LOG"))));

    AppTracing::new(console_filter, file)
  }
}

pub struct AppTracing {
  _file_tracing: FileTracing,
}
#[cfg(feature = "app_tracing_file")]
#[derive(Default)]
struct FileTracing(Option<tracing_appender::non_blocking::WorkerGuard>);
#[cfg(not(feature = "app_tracing_file"))]
#[derive(Default)]
struct FileTracing;


impl AppTracing {
  fn new(
    console_filter: EnvFilter,
    file: Option<(&Path, EnvFilter)>,
  ) -> Self {
    #[cfg(not(target_arch = "wasm32"))] {
      Self::create(console_filter, file)
    }
    #[cfg(target_arch = "wasm32")] {
      Self::create_wasm(console_filter)
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn create(
    console_filter: EnvFilter,
    file: Option<(&Path, EnvFilter)>,
  ) -> Self {
    let layered = tracing_subscriber::registry();
    let layered = layered.with(
      tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .with_filter(console_filter)
    );

    let _file_tracing = if let Some((file_path, filter)) = file {
      let result = (|| {
        if let Some(parent) = file_path.parent() {
          create_dir_all(parent)?;
        }
        File::create(file_path)
      })();
      match result {
        Err(e) => {
          layered.init();
          warn!("Cannot log to file; could not truncate/create and open log file '{}' for writing: {}", file_path.display(), e);
          FileTracing::default()
        }
        Ok(log_file) => {
          let writer = BufWriter::new(log_file);
          let (non_blocking, guard) = tracing_appender::non_blocking(writer);
          let layered = layered.with(
            tracing_subscriber::fmt::layer()
              .with_writer(non_blocking)
              .with_ansi(false)
              .with_filter(filter)
          );
          layered.init();
          FileTracing(Some(guard))
        }
      }
    } else {
      layered.init();
      FileTracing::default()
    };

    Self { _file_tracing }
  }

  #[cfg(target_arch = "wasm32")]
  fn create_wasm(
    console_filter: EnvFilter,
  ) -> Self {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let layered = tracing_subscriber::registry();
    let layered = layered.with(
      tracing_subscriber::fmt::layer()
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_ansi(false)
        .without_time()
        .with_filter(console_filter)
    );
    layered.init();

    let _file_tracing = FileTracing::default();
    Self { _file_tracing }
  }
}

