use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;

use att_core::{Crate, Search};

use crate::data::Data;
use crate::krate::crates_io_client::CratesIoClient;

#[derive(Clone)]
pub struct App {
  data: Arc<RwLock<Data>>,
  crates_io_client: CratesIoClient,
}

impl App {
  pub fn new(
    data: Arc<RwLock<Data>>,
    crates_io_client: CratesIoClient,
  ) -> Self {
    Self {
      data,
      crates_io_client,
    }
  }
}

pub async fn run(app: App, shutdown_signal: impl Future<Output=()> + Send + 'static) -> Result<(), Box<dyn Error>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
  let listener = tokio::net::TcpListener::bind(addr).await?;

  use axum::routing::{get, post};
  let router = Router::new()
    .route("/crate/blessed", get(crate_blessed))
    .route("/crate/blessed/add/:id", post(crate_blessed_add))
    .route("/crate/blessed/remove/:id", post(crate_blessed_remove))
    .route("/crate/search", get(crate_search))
    .route("/crate/refresh/outdated", post(crate_refresh_outdated))
    .route("/crate/refresh/all", post(crate_refresh_all))
    .route("/crate/refresh/one/:id", post(crate_refresh_one))
    .with_state(app)
    ;

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal)
    .await?;

  Ok(())
}

async fn crate_blessed(State(app): State<App>) -> Json<Vec<Crate>> {
  let data = app.data.read().await;
  let blessed_crates = data.crate_data.get_blessed_crates();
  Json(blessed_crates)
}
async fn crate_blessed_add(State(app): State<App>, Path(id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = data.crate_data.add_blessed_crate(id, &app.crates_io_client).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn crate_blessed_remove(State(app): State<App>, Path(id): Path<String>) {
  let mut data = app.data.write().await;
  data.crate_data.remove_blessed_crate(id);
}

async fn crate_search(State(app): State<App>, Json(search): Json<Search>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = app.data.write().await;
  let crates = data.crate_data.search(search, &app.crates_io_client).await.map_err(|_| F)?;
  Ok(Json(crates))
}

async fn crate_refresh_outdated(State(app): State<App>) -> Result<(), F> {
  let mut data = app.data.write().await;
  data.crate_data.refresh_outdated(&app.crates_io_client).await.map_err(|_| F)?;
  Ok(())
}
async fn crate_refresh_all(State(app): State<App>) -> Result<(), F> {
  let mut data = app.data.write().await;
  data.crate_data.refresh_all(&app.crates_io_client).await.map_err(|_| F)?;
  Ok(())
}
async fn crate_refresh_one(State(app): State<App>, Path(id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = data.crate_data.refresh_one(id, &app.crates_io_client).await.map_err(|_| F)?;
  Ok(Json(krate))
}

// Error "utility"
struct F;
impl IntoResponse for F {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
  }
}
