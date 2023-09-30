use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CrateResponse, CratesPage, CratesQuery, Sort};
use iced::futures::future::{Either, pending};
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
  pub async fn update(self, crate_name: String) -> Result<UpdateResponse, AsyncError> {
    Ok(self.send_receive(|tx| Request::Update(crate_name, tx)).await?)
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
}

enum Request {
  Search(String, oneshot::Sender<SearchResponse>),
  CancelSearch,
  Update(String, oneshot::Sender<UpdateResponse>)
}

impl Manager {
  async fn run(mut self) {
    let mut queued_updates = VecDeque::new();

    pin! {
      let search = Either::Left(pending());
      let update = Either::Left(pending());
    }
    let mut running_search = false;
    let mut running_update = false;

    loop {
      select! {
        _ = &mut search => {
          search.set(Either::Left(pending()));
          running_search = false;
          if !running_update {
            // TODO: remove code duplication
            if let Some((crate_name, tx)) = queued_updates.pop_front() {
              update.set(Either::Right(do_update(crate_name, self.client.clone(), tx)));
              running_update = true;
            }
          }
        },
        _ = &mut update => {
          update.set(Either::Left(pending()));
          running_update = false;
          if !running_search {
            // TODO: remove code duplication
            if let Some((crate_name, tx)) = queued_updates.pop_front() {
              update.set(Either::Right(do_update(crate_name, self.client.clone(), tx)));
              running_update = true;
            }
          }
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search_term, tx) => {
              let sleep_until = Instant::now() + Duration::from_millis(300);
              search.set(Either::Right(do_search(sleep_until, search_term, self.client.clone(), tx)));
              running_search = true;
            },
            Request::CancelSearch => {
              search.set(Either::Left(pending()));
              running_search = false;

              if !running_update {
                // TODO: remove code duplication
                if let Some((crate_name, tx)) = queued_updates.pop_front() {
                  update.set(Either::Right(do_update(crate_name, self.client.clone(), tx)));
                  running_update = true;
                }
              }
            },
            Request::Update(crate_name, tx) => {
              if running_search || running_update {
                queued_updates.push_back((crate_name, tx));
              } else if queued_updates.is_empty() {
                update.set(Either::Right(do_update(crate_name, self.client.clone(), tx)));
                running_update = true;
              } else {
                queued_updates.push_back((crate_name, tx));
              }
            }
          }
        },
        else => { break; }
      }
    }
  }
}

async fn do_search(sleep_until: Instant, search_term: String, client: AsyncClient, tx: oneshot::Sender<SearchResponse>) {
  tokio::time::sleep_until(sleep_until.into()).await;
  let query = CratesQuery::builder()
    .search(search_term)
    .sort(Sort::Relevance)
    .build();
  let response = client.crates(query).await;
  let _ = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
}
async fn do_update(crate_name: String, client: AsyncClient, tx: oneshot::Sender<UpdateResponse>) {
  let response = client.get_crate(&crate_name).await;
  let _ = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
}
