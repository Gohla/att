use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CratesPage, CratesQuery, Sort};
use iced::futures::future::{Either, pending};
use thiserror::Error;
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

impl CratesClient {
  pub fn new(crates_io_api: AsyncClient) -> Self {
    let (tx, rx) = mpsc::channel(64);
    let manager = Manager { crates_io_api, rx };
    tokio::spawn(manager.run());
    Self { tx }
  }


  pub async fn search(self, search_term: String) -> Result<Result<CratesPage, crates_io_api::Error>, AsyncError> {
    Ok(self.send_receive(|tx| Request::Search(search_term, tx)).await?)
  }

  pub async fn cancel_search(self) -> Result<(), AsyncError> {
    self.send(Request::CancelSearch).await
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
  crates_io_api: AsyncClient,
  rx: mpsc::Receiver<Request>,
}

enum Request {
  Search(String, oneshot::Sender<Result<CratesPage, crates_io_api::Error>>),
  CancelSearch,
}

impl Manager {
  async fn run(mut self) {
    let search = Either::Left(pending());
    tokio::pin!(search);

    loop {
      tokio::select! {
        _ = &mut search => {
          search.set(Either::Left(pending()));
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search_term, tx) => {
              let sleep_until = Instant::now() + Duration::from_millis(300);
              search.set(Either::Right(do_search(sleep_until, search_term, self.crates_io_api.clone(), tx)));
            },
            Request::CancelSearch => {
              search.set(Either::Left(pending()));
            }
          }
        },
        else => { break; }
      }
    }
  }
}

async fn do_search(
  sleep_until: Instant,
  search_term: String,
  crates_io_api: AsyncClient,
  tx: oneshot::Sender<Result<CratesPage, crates_io_api::Error>>,
) {
  tokio::time::sleep_until(sleep_until.into()).await;
  let query = CratesQuery::builder()
    .search(search_term)
    .sort(Sort::Relevance)
    .build();
  let response = crates_io_api.crates(query).await;
  let _ = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
}
