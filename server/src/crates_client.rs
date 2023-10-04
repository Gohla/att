use std::collections::VecDeque;
use std::error::Error;
use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CrateResponse, CratesPage, CratesQuery, Sort};
use futures::future::{BoxFuture, Fuse, FusedFuture};
use futures::FutureExt;
use tokio::sync::{mpsc, oneshot};

use crate::async_util::AsyncError;

#[derive(Clone)]
pub struct CratesClient {
  tx: mpsc::Sender<Request>
}

pub type SearchResponse = Result<CratesPage, crates_io_api::Error>;
pub type RefreshResponse = Result<CrateResponse, crates_io_api::Error>;

impl CratesClient {
  pub fn new(user_agent: &str) -> Result<Self, Box<dyn Error>> {
    let client = AsyncClient::new(user_agent, Duration::from_secs(1))?;
    let (tx, rx) = mpsc::channel(64);
    let manager = Task::new(rx, client);
    tokio::spawn(manager.run());
    Ok(Self { tx })
  }

  pub async fn search(self, wait_until: Instant, search_term: String) -> Result<SearchResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Search(Search { wait_until, search_term, tx })).await?)
  }
  pub async fn cancel_search(self) -> Result<(), AsyncError> {
    self.send(Request::CancelSearch).await
  }
  pub async fn refresh(self, id: String) -> Result<RefreshResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Refresh(Refresh { id, tx })).await?)
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
    tracing::info!("crates manager task is ending");
  }

  fn run_search(&mut self, search: Search) {
    self.search = search.run(self.client.clone()).boxed().fuse();
  }
  fn cancel_search(&mut self) {
    tracing::info!("cancelling crate search");
    self.search = Fuse::terminated();
  }

  fn queue_refresh(&mut self, refresh: Refresh) {
    tracing::info!(id = refresh.id, "queueing crate refresh");
    self.queue.push_back(refresh);
  }
  fn try_run_queued_refresh(&mut self) {
    if self.search.is_terminated() && self.refresh.is_terminated() {
      if let Some(refresh) = self.queue.pop_front() {
        tracing::info!(id = refresh.id, "dequeued crate refresh");
        self.run_refresh(refresh);
      }
    }
  }
  fn run_refresh(&mut self, refresh: Refresh) {
    self.refresh = refresh.run(self.client.clone()).boxed().fuse();
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

struct Refresh {
  id: String,
  tx: oneshot::Sender<RefreshResponse>,
}
impl Refresh {
  async fn run(self, client: AsyncClient) {
    tracing::info!(id = self.id, "running crate refresh");
    let response = client.get_crate(&self.id).await;
    let _ = self.tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  }
}
