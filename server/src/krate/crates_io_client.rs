#![allow(dead_code)]

use std::collections::VecDeque;
use std::error::Error;
use std::time::Duration;

use crates_io_api::{AsyncClient, CrateResponse, CratesPage, CratesQuery, Sort};
use futures::future::{BoxFuture, Fuse, FusedFuture};
use futures::FutureExt;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct CratesIoClient {
  tx: mpsc::Sender<Request>
}

#[derive(Debug, thiserror::Error)]
pub enum CratesIoClientError {
  #[error("Failed to execute request: {0}")]
  CratesIo(#[from] crates_io_api::Error),
  #[error("Failed to send request; receiver was closed")]
  Tx,
  #[error("Failed to receive response; sender was closed")]
  Rx,
}
impl<T> From<mpsc::error::SendError<T>> for CratesIoClientError {
  fn from(_: mpsc::error::SendError<T>) -> Self { Self::Tx }
}
impl From<oneshot::error::RecvError> for CratesIoClientError {
  fn from(_: oneshot::error::RecvError) -> Self { Self::Rx }
}


impl CratesIoClient {
  pub fn new(user_agent: &str) -> Result<Self, Box<dyn Error>> {
    let client = AsyncClient::new(user_agent, Duration::from_secs(1))?;
    let (tx, rx) = mpsc::channel(64);
    let manager = Task::new(rx, client);
    tokio::spawn(manager.run());
    Ok(Self { tx })
  }

  pub async fn search(self, search_term: String) -> Result<CratesPage, CratesIoClientError> {
    self.send_receive(|tx| Request::Search(Search { search_term, tx })).await
  }
  pub async fn cancel_search(self) -> Result<(), CratesIoClientError> {
    self.send(Request::CancelSearch).await
  }

  pub async fn refresh(self, crate_id: String) -> Result<CrateResponse, CratesIoClientError> {
    self.send_receive(|tx| Request::Refresh(Refresh { crate_id, tx })).await
  }

  async fn send_receive<T>(&self, make_request: impl FnOnce(oneshot::Sender<Result<T, crates_io_api::Error>>) -> Request) -> Result<T, CratesIoClientError> {
    let (tx, rx) = oneshot::channel();
    let request = make_request(tx);
    self.tx.send(request).await?;
    let response = rx.await??;
    Ok(response)
  }
  async fn send(&self, request: Request) -> Result<(), CratesIoClientError> {
    self.tx.send(request).await?;
    Ok(())
  }
}


enum Request {
  Search(Search),
  CancelSearch,
  Refresh(Refresh),
}

struct Task {
  rx: mpsc::Receiver<Request>,
  client: AsyncClient,
  search: Fuse<BoxFuture<'static, ()>>,
  refresh: Fuse<BoxFuture<'static, ()>>,
  queue: VecDeque<Refresh>,
}
impl Task {
  fn new(rx: mpsc::Receiver<Request>, client: AsyncClient) -> Self {
    let task = Self {
      rx,
      client,
      queue: VecDeque::new(),
      search: Fuse::terminated(),
      refresh: Fuse::terminated()
    };
    task
  }

  #[tracing::instrument(skip_all)]
  async fn run(mut self) {
    loop {
      tokio::select! {
        _ = &mut self.search => {
          self.try_run_queued_refresh();
        },
        _ = &mut self.refresh => {
          self.try_run_queued_refresh();
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search) => {
              self.run_search(search);
            },
            Request::CancelSearch => {
              self.cancel_search();
              self.try_run_queued_refresh();
            },
            Request::Refresh(refresh) => {
              self.queue_refresh(refresh);
              self.try_run_queued_refresh();
            },
          }
        },
        else => { break; }
      }
    }
    tracing::info!("crates-io-client task is ending");
  }

  fn run_search(&mut self, search: Search) {
    tracing::trace!(search_term = search.search_term, "starting crate search");
    self.search = search.run(self.client.clone()).boxed().fuse();
  }
  fn cancel_search(&mut self) {
    tracing::trace!("cancelling crate search");
    self.search = Fuse::terminated();
  }

  fn queue_refresh(&mut self, refresh: Refresh) {
    tracing::trace!(id = refresh.crate_id, "queueing crate refresh");
    self.queue.push_back(refresh);
  }
  fn try_run_queued_refresh(&mut self) {
    if self.search.is_terminated() && self.refresh.is_terminated() {
      if let Some(refresh) = self.queue.pop_front() {
        tracing::info!(id = refresh.crate_id, "dequeued crate refresh");
        self.run_refresh(refresh);
      }
    }
  }
  fn run_refresh(&mut self, refresh: Refresh) {
    tracing::trace!(crate_id = refresh.crate_id, "starting crate refresh");
    self.refresh = refresh.run(self.client.clone()).boxed().fuse();
  }
}

struct Search {
  search_term: String,
  tx: oneshot::Sender<Result<CratesPage, crates_io_api::Error>>,
}
impl Search {
  async fn run(self, client: AsyncClient) {
    tracing::info!(search_term = self.search_term, "running crate search");
    let query = CratesQuery::builder()
      .search(self.search_term)
      .sort(Sort::Relevance)
      .build();
    let response = client.crates(query).await;
    let _ = self.tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  }
}

struct Refresh {
  crate_id: String,
  tx: oneshot::Sender<Result<CrateResponse, crates_io_api::Error>>,
}
impl Refresh {
  async fn run(self, client: AsyncClient) {
    tracing::info!(crate_id = self.crate_id, "running crate refresh");
    let response = client.get_crate(&self.crate_id).await;
    let _ = self.tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  }
}
