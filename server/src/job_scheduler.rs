#![allow(dead_code)]

use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio::task::{block_in_place, JoinError, JoinSet};
use tokio::time::Interval;
use tracing::{debug, error, info};

// Public API

pub struct JobScheduler {
  tx: mpsc::Sender<Request>,
}
impl JobScheduler {
  pub fn new() -> (Self, impl Future<Output=()>) {
    let (tx, rx) = mpsc::channel(64);
    let task = Task::new(rx).run();
    (Self { tx }, task)
  }
  pub fn blocking_schedule_job(&self, job: impl Job, interval: Interval, name: impl Into<String>) {
    let _ = self.tx.blocking_send(Request::ScheduleJob(Box::new(job), interval, name.into()));
  }
  pub async fn schedule_job(&self, job: impl Job, interval: Interval, name: impl Into<String>) {
    let _ = self.tx.send(Request::ScheduleJob(Box::new(job), interval, name.into())).await;
  }

  pub fn blocking_schedule_blocking_job(&self, job: impl BlockingJob, interval: Interval, name: impl Into<String>) {
    let _ = self.tx.blocking_send(Request::ScheduleBlockingJob(Box::new(job), interval, name.into()));
  }
  pub async fn schedule_blocking_job(&self, job: impl BlockingJob, interval: Interval, name: impl Into<String>) {
    let _ = self.tx.send(Request::ScheduleBlockingJob(Box::new(job), interval, name.into())).await;
  }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum JobAction {
  #[default]
  Continue,
  Cancel
}
pub type JobResult = Result<JobAction, Box<dyn Error + Send + Sync + 'static>>;

pub trait Job: Send + 'static {
  fn run(&self) -> impl Future<Output=JobResult> + Send;
}
pub trait BlockingJob: Send + 'static {
  fn run(&self) -> JobResult;
}


// Internals

trait JobDyn: Send {
  fn run(&self) -> Pin<Box<dyn Future<Output=JobResult> + Send + '_>>;
}
impl<T: Job> JobDyn for T {
  fn run(&self) -> Pin<Box<dyn Future<Output=JobResult> + Send + '_>> { Box::pin(<Self as Job>::run(self)) }
}

enum Request {
  ScheduleJob(Box<dyn JobDyn>, Interval, String),
  ScheduleBlockingJob(Box<dyn BlockingJob>, Interval, String),
}

struct Task {
  rx: mpsc::Receiver<Request>,
  jobs: JoinSet<String>,
}
impl Task {
  fn new(rx: mpsc::Receiver<Request>) -> Self {
    let task = Self {
      rx,
      jobs: Default::default(),
    };
    task
  }

  //noinspection RsBorrowChecker
  async fn run(mut self) {
    loop {
      tokio::select! {
        o = self.rx.recv() => match o {
          Some(request) => self.handle_request(request),
          None => break,
        },
        Some(job_join_result) = self.jobs.join_next() => Self::handle_job_complete(job_join_result),
        else => break,
      }
    }

    debug!("job scheduler task is stopping");
    self.jobs.shutdown().await;
  }

  fn handle_request(&mut self, request: Request) {
    match request {
      Request::ScheduleJob(job, interval, name) => {
        info!("registering job '{}' at interval: {:?}", name, interval.period());
        self.jobs.spawn(Self::run_job(job, interval, name));
      },
      Request::ScheduleBlockingJob(job, interval, name) => {
        info!("registering blocking job '{}' at interval: {:?}", name, interval.period());
        self.jobs.spawn(Self::run_blocking_job(job, interval, name));
      }
    }
  }
  async fn run_job(job: Box<dyn JobDyn>, mut interval: Interval, name: String) -> String {
    loop {
      interval.tick().await;
      info!("running job: {}", name);
      let job_result = job.run().await;
      if Self::handle_job_result(job_result, &name) {
        return name;
      }
    }
  }
  async fn run_blocking_job(job: Box<dyn BlockingJob>, mut interval: Interval, name: String) -> String {
    loop {
      interval.tick().await;
      info!("running blocking job: {}", name);
      let job_result = block_in_place(|| job.run());
      if Self::handle_job_result(job_result, &name) {
        return name;
      }
    }
  }
  fn handle_job_result(result: JobResult, name: &str) -> bool {
    match result {
      Ok(action) => {
        info!("job '{}' was executed successfully", name);
        match action {
          JobAction::Cancel => {
            info!("job '{}' requested to be cancelled", name);
            return true;
          },
          JobAction::Continue => {}
        }
      }
      Err(cause) => error!(?cause, "job '{}' was executed unsuccessfully", name),
    }
    false
  }

  fn handle_job_complete(result: Result<String, JoinError>) {
    match result {
      Err(join_error) => {
        if let Ok(panic) = join_error.try_into_panic() {
          error!(?panic, "a job has panicked");
        } else {
          info!("a job was cancelled");
        }
      }
      Ok(name) => {
        info!("job '{}' has been cancelled", name);
      }
    }
  }
}
