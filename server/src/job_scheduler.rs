#![allow(dead_code)]

use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::Interval;

// Public API

#[derive(Clone)]
pub struct JobScheduler {
  tx: mpsc::Sender<Request>
}
impl JobScheduler {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel(64);
    let task = Task::new(rx);
    tokio::spawn(task.run());
    Self { tx }
  }

  pub fn schedule(&self, interval: Interval, job: impl Job) {
    let _ = self.tx.blocking_send(Request::Schedule(interval, Box::new(job)));
  }
  pub async fn schedule_async(self, interval: Interval, job: impl Job) {
    let _ = self.tx.send(Request::Schedule(interval, Box::new(job))).await;
  }
}

pub type JobOutput = Result<(), Box<dyn Error + Send + Sync + 'static>>;

pub trait Job: Send + 'static {
  fn run(&self) -> impl Future<Output=JobOutput> + Send;
}


// Internals

trait JobDyn: Send {
  fn run(&self) -> Pin<Box<dyn Future<Output=JobOutput> + Send + '_>>;
}
impl<T: Job> JobDyn for T {
  fn run(&self) -> Pin<Box<dyn Future<Output=JobOutput> + Send + '_>> {
    Box::pin(<Self as Job>::run(self))
  }
}

enum Request {
  Schedule(Interval, Box<dyn JobDyn>),
}

struct Task {
  rx: mpsc::Receiver<Request>,
  jobs: JoinSet<JobOutput>,
}
impl Task {
  fn new(rx: mpsc::Receiver<Request>) -> Self {
    let task = Self {
      rx,
      jobs: Default::default(),
    };
    task
  }

  #[tracing::instrument(skip_all)]
  async fn run(mut self) {
    loop {
      tokio::select! {
        Some(join_result) = self.jobs.join_next() => {
          match join_result {
            Err(join_error) => {
              if let Ok(panic) = join_error.try_into_panic() {
                tracing::error!(?panic, "a job has panicked");
              } else {
                tracing::info!("a job was cancelled");
              }
            }
            Ok(Ok(())) => {
              tracing::info!("a job has completed successfully");
            }
            Ok(Err(cause)) => {
              tracing::error!(?cause, "a job has completed unsuccessfully");
            }
          }
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Schedule(mut interval, job) => {
              self.jobs.spawn(async move {
                loop {
                  interval.tick().await;
                  match job.run().await {
                    Ok(()) => tracing::info!("a job was executed successfully"),
                    Err(cause) => tracing::error!(?cause, "a job was executed unsuccessfully"),
                  }
                }
              });
            },
          }
        },
        else => { break; }
      }
    }

    tracing::info!("job scheduler task is ending");
  }
}
