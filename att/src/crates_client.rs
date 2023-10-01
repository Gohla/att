use std::collections::VecDeque;
use std::time::Instant;

use crates_io_api::{AsyncClient, CrateResponse, CratesPage, CratesQuery, Sort};
use iced::futures::future::{BoxFuture, Fuse, FusedFuture};
use iced::futures::FutureExt;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct CratesClient {
  tx: mpsc::Sender<Request>
}

#[derive(Debug, thiserror::Error)]
pub enum AsyncError {
  #[error("Failed to send request; manager receiver was closed")]
  Tx,
  #[error("Failed to receive response; sender was closed")]
  Rx,
}
impl<T> From<mpsc::error::SendError<T>> for AsyncError {
  fn from(_: mpsc::error::SendError<T>) -> Self { Self::Tx }
}
impl From<oneshot::error::RecvError> for AsyncError {
  fn from(_: oneshot::error::RecvError) -> Self { Self::Rx }
}

pub type SearchResponse = Result<CratesPage, crates_io_api::Error>;
pub type UpdateResponse = Result<CrateResponse, crates_io_api::Error>;

impl CratesClient {
  pub fn new(client: AsyncClient) -> Self {
    let (tx, rx) = mpsc::channel(64);
    let manager = Manager::new(client, rx);
    tokio::spawn(manager.run());
    Self { tx }
  }

  pub async fn search(self, wait_until: Instant, search_term: String) -> Result<SearchResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Search(Search { wait_until, search_term, tx })).await?)
  }
  pub async fn cancel_search(self) -> Result<(), AsyncError> {
    self.send(Request::CancelSearch).await
  }
  pub async fn update(self, id: String) -> Result<UpdateResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Update(Update { id, tx })).await?)
  }

  async fn send_receive<T>(&self, make_request: impl FnOnce(oneshot::Sender<T>) -> Request) -> Result<T, AsyncError> {
    let (tx, rx) = oneshot::channel();
    self.tx.send(make_request(tx)).await?;
    Ok(rx.await?)
  }
  async fn send(&self, request: Request) -> Result<(), AsyncError> {
    self.tx.send(request).await?;
    Ok(())
  }
}


enum Request {
  Search(Search),
  CancelSearch,
  Update(Update)
}

struct Manager {
  client: AsyncClient,
  rx: mpsc::Receiver<Request>,
  running_search: Fuse<BoxFuture<'static, ()>>,
  running_update: Fuse<BoxFuture<'static, ()>>,
  queued_updates: VecDeque<Update>
}
impl Manager {
  fn new(client: AsyncClient, rx: mpsc::Receiver<Request>) -> Self {
    Self {
      client,
      rx,
      queued_updates: VecDeque::new(),
      running_search: Fuse::terminated(),
      running_update: Fuse::terminated()
    }
  }

  #[tracing::instrument(skip_all)]
  async fn run(mut self) {
    loop {
      tokio::select! {
        _ = &mut self.running_search => {
          if self.running_update.is_terminated() {
            self.try_run_queued_update();
          }
        },
        _ = &mut self.running_update => {
          if self.running_search.is_terminated() {
            self.try_run_queued_update();
          }
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search) => {
              self.run_search(search);
            },
            Request::CancelSearch => {
              self.cancel_search();
              if self.running_update.is_terminated() {
                self.try_run_queued_update();
              }
            },
            Request::Update(update) => {
              if !self.running_search.is_terminated() || !self.running_update.is_terminated() {
                self.queue_update(update);
              } else if self.queued_updates.is_empty() {
                self.run_update(update);
              } else {
                self.queue_update(update);
              }
            }
          }
        },
        else => { break; }
      }
    }
    tracing::info!("crates manager task is ending");
  }

  fn run_search(&mut self, search: Search) {
    self.running_search = search.run(self.client.clone()).boxed().fuse();
  }
  fn cancel_search(&mut self) {
    tracing::info!("cancelling crate search");
    self.running_search = Fuse::terminated();
  }

  fn queue_update(&mut self, update: Update) {
    tracing::info!(id = update.id, "queueing crate update");
    self.queued_updates.push_back(update);
  }
  fn try_run_queued_update(&mut self) {
    if let Some(update) = self.queued_updates.pop_front() {
      tracing::info!(id = update.id, "dequeued crate update");
      self.run_update(update);
    }
  }
  fn run_update(&mut self, update: Update) {
    self.running_update = update.run(self.client.clone()).boxed().fuse();
  }
}

struct Search {
  wait_until: Instant,
  search_term: String,
  tx: oneshot::Sender<SearchResponse>,
}
impl Search {
  async fn run(self, client: AsyncClient) {
    tracing::info!(wait_until = ?self.wait_until, search_term = self.search_term, "running crate search");
    tokio::time::sleep_until(self.wait_until.into()).await;
    let query = CratesQuery::builder()
      .search(self.search_term)
      .sort(Sort::Relevance)
      .build();
    let response = client.crates(query).await;
    let _ = self.tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  }
}

struct Update {
  id: String,
  tx: oneshot::Sender<UpdateResponse>,
}
impl Update {
  async fn run(self, client: AsyncClient) {
    tracing::info!(id = self.id, "running crate update");
    let response = client.get_crate(&self.id).await;
    let _ = self.tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  }
}
