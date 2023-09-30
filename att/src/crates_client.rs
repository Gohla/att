use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CratesPage, CratesQuery, Sort};
use iced::futures::future::OptionFuture;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct CratesClient {
  tx: mpsc::Sender<Request>
}

impl CratesClient {
  pub fn new(crates_io_api: AsyncClient) -> Self {
    let (tx, rx) = mpsc::channel(100);
    let manager = Manager { crates_io_api, rx };
    tokio::spawn(manager.run());
    Self { tx }
  }

  pub async fn search(self, search_term: String) -> Option<Result<CratesPage, crates_io_api::Error>> {
    let (tx, rx) = oneshot::channel();
    if self.tx.send(Request::Search(search_term, tx)).await.is_err() {
      println!("Client: Manager receiver was closed");
      return None;
    } else {
      println!("Client: Sent");
    }
    let result = rx.await;
    if let Err(_) = result {
      println!("Client: Oneshot sender was closed");
    } else {
      println!("Client: Received");
    }
    result.ok()
  }
}


struct Manager {
  crates_io_api: AsyncClient,
  rx: mpsc::Receiver<Request>,
}

enum Request {
  Search(String, oneshot::Sender<Result<CratesPage, crates_io_api::Error>>),
}

impl Manager {
  async fn run(mut self) {
    let search: OptionFuture<_> = None.into();
    tokio::pin!(search);

    loop {
      tokio::select! {
        Some(_) = &mut search => {
          println!("Manager: Setting search to None because search completed");
          search.set(None.into());
        },
        Some(request) = self.rx.recv() => {
          match request {
            Request::Search(search_term, tx) => {
              if search_term.is_empty() {
                println!("Manager: Setting search to None because search term is empty");
                search.set(None.into());
              } else {
                println!("Manager: Setting search to Some because search term is not empty");
                let sleep_until = Instant::now() + Duration::from_millis(1000);
                search.set(Some(do_search(sleep_until, search_term, self.crates_io_api.clone(), tx)).into());
              }
            }
          }
        },
        else => {
          println!("Manager: Breaking out!");
          break;
        }
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
  println!("Manager: Sleeping until {:?}", sleep_until);
  tokio::time::sleep_until(sleep_until.into()).await;
  let query = CratesQuery::builder()
    .search(search_term)
    .sort(Sort::Relevance)
    .build();
  println!("Manager: Searching");
  let response = crates_io_api.crates(query).await;
  println!("Manager: Response {:?}", response);
  let result = tx.send(response); // Ignore error ok: do nothing if receiver was dropped.
  if result.is_err() {
    println!("Manager: Send fail");
  } else {
    println!("Manager: Send success");
  }
}
