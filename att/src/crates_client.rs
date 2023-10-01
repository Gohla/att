use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CrateResponse, CratesPage, CratesQuery, Sort};
use iced::futures::future::{Fuse, FusedFuture};
use iced::futures::FutureExt;
use thiserror::Error;
use tokio::{pin, select};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct CratesClient {
  tx: mpsc::Sender<Request>
}

#[derive(Debug, Error)]
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
    let manager = Manager { client, rx };
    tokio::spawn(manager.run());
    Self { tx }
  }

  pub async fn search(self, search_term: String) -> Result<SearchResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Search(search_term, tx)).await?)
  }
  pub async fn cancel_search(self) -> Result<(), AsyncError> {
    self.send(Request::CancelSearch).await
  }
  pub async fn update(self, id: String) -> Result<UpdateResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Update(id, tx)).await?)
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


struct Manager {
  client: AsyncClient,
  rx: mpsc::Receiver<Request>,

  // running_search: bool,
  // running_update: bool,
}

enum Request {
  Search(String, oneshot::Sender<SearchResponse>),
  CancelSearch,
  Update(String, oneshot::Sender<UpdateResponse>)
}

impl Manager {
  async fn run(mut self) {
    let mut queued_updates = VecDeque::new();

    let search = Fuse::terminated();
    let update = Fuse::terminated();
    pin!(search, update);

    loop {
      select! {
        _ = &mut search => {
          if update.is_terminated() {
            // TODO: remove code duplication
            if let Some((id, tx)) = queued_updates.pop_front() {
              tracing::info!(id, "dequeued and starting crate update");
              update.set(do_update(id, self.client.clone(), tx).fuse());
            }
          }
        },
        _ = &mut update => {
          if search.is_terminated() {
            // TODO: remove code duplication
            if let Some((id, tx)) = queued_updates.pop_front() {
              tracing::info!(id, "dequeued and starting crate update");
              update.set(do_update(id, self.client.clone(), tx).fuse());
            }
          }
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search_term, tx) => {
              let sleep_until = Instant::now() + Duration::from_millis(300);
              tracing::info!(?sleep_until, search_term, "starting crate search");
              search.set(do_search(sleep_until, search_term, self.client.clone(), tx).fuse());
            },
            Request::CancelSearch => {
              tracing::info!("cancelling crate search");
              search.set(Fuse::terminated());

              if update.is_terminated() {
                // TODO: remove code duplication
                if let Some((id, tx)) = queued_updates.pop_front() {
                  tracing::info!(id, "dequeued and starting crate update");
                  update.set(do_update(id, self.client.clone(), tx).fuse());
                }
              }
            },
            Request::Update(id, tx) => {
              if !search.is_terminated() || !update.is_terminated() {
                tracing::info!(id, "queueing crate update");
                queued_updates.push_back((id, tx));
              } else if queued_updates.is_empty() {
                tracing::info!(id, "starting crate update");
                update.set(do_update(id, self.client.clone(), tx).fuse());
              } else {
                tracing::info!(id, "queueing crate update");
                queued_updates.push_back((id, tx));
              }
            }
          }
        },
        else => { break; }
      }
    }
  }

  // fn perform_search(&mut self, search_term: String, tx: oneshot::Sender<SearchResponse>) {
  //   let sleep_until = Instant::now() + Duration::from_millis(300);
  //   tracing::info!(?sleep_until, search_term, "starting crate search");
  //   search.set(Either::Right(do_search(sleep_until, search_term, self.client.clone(), tx)));
  //   self.running_search = true;
  // }
}

#[tracing::instrument(skip(client, tx))]
async fn do_search(sleep_until: Instant, search_term: String, client: AsyncClient, tx: oneshot::Sender<SearchResponse>) {
  tokio::time::sleep_until(sleep_until.into()).await;
  let query = CratesQuery::builder()
    .search(search_term)
    .sort(Sort::Relevance)
    .build();
  let response = client.crates(query).await;
  let _ = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
}

#[tracing::instrument(skip(client, tx))]
async fn do_update(id: String, client: AsyncClient, tx: oneshot::Sender<UpdateResponse>) {
  let response = client.get_crate(&id).await;
  let _ = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
}
