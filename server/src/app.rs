use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router, routing::get};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;

use att_core::Crate;

use crate::data::Data;
use crate::krate::crates_io_client::CratesIoClient;

#[derive(Clone)]
pub struct App {
  data: Arc<RwLock<Data>>,
  crates_client: CratesIoClient,
}

impl App {
  pub fn new(
    data: Arc<RwLock<Data>>,
    crates_client: CratesIoClient,
  ) -> Self {
    Self {
      data,
      crates_client,
    }
  }
}

pub async fn run(app: App, shutdown_signal: impl Future<Output=()>) -> Result<(), Box<dyn Error>> {
  let router = Router::new()
    .route("/crate/blessed", get(crate_blessed))
    .route("/crate/search/:term", get(crate_search))
    .route("/crate/refresh/outdated", get(crate_refresh_outdated))
    .route("/crate/refresh/all", get(crate_refresh_all))
    .route("/crate/refresh/:id", get(crate_refresh))
    .with_state(app)
    ;

  let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
  axum::Server::bind(&addr)
    .serve(router.into_make_service())
    .with_graceful_shutdown(shutdown_signal)
    .await?;

  Ok(())
}

async fn crate_blessed(State(app): State<App>) -> Json<Vec<Crate>> {
  let data = app.data.read().await;
  let blessed_crates = data.crate_data.blessed_crates();
  Json(blessed_crates)
}

async fn crate_search(State(app): State<App>, Path(term): Path<String>) -> Result<Json<Vec<Crate>>, F> {
  let response = app.crates_client.search(term).await.map_err(|_| F)?.map_err(|_| F)?;
  let crates: Vec<_> = response.crates.into_iter().map(|c| c.into()).collect();
  Ok(Json(crates))
}

async fn crate_refresh(State(app): State<App>, Path(id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = data.crate_data.refresh(id, &app.crates_client).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn crate_refresh_outdated(State(app): State<App>) -> Result<(), F> {
  let mut data = app.data.write().await;
  data.crate_data.refresh_outdated_data(&app.crates_client).await.map_err(|_| F)?;
  Ok(())
}
async fn crate_refresh_all(State(app): State<App>) -> Result<(), F> {
  let mut data = app.data.write().await;
  data.crate_data.refresh_all_data(&app.crates_client).await.map_err(|_| F)?;
  Ok(())
}


// Error "utility"
struct F;
impl IntoResponse for F {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
  }
}
