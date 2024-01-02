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
use crate::krate::Crates;

#[derive(Clone)]
pub struct App {
  data: Arc<RwLock<Data>>,
  crates: Crates,
}

impl App {
  pub fn new(
    data: Arc<RwLock<Data>>,
    crates: Crates,
  ) -> Self {
    Self {
      data,
      crates,
    }
  }
}

pub async fn run(app: App, shutdown_signal: impl Future<Output=()> + Send + 'static) -> Result<(), Box<dyn Error>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
  let listener = tokio::net::TcpListener::bind(addr).await?;

  use axum::routing::{get, post};
  let router = Router::new()
    .route("/api/v1/crates", get(search_crates))
    .route("/api/v1/crates/:crate_id", get(get_crate))
    .route("/api/v1/crates/:crate_id/follow", post(follow_crate).delete(unfollow_crate))
    .route("/api/v1/crates/:crate_id/refresh", post(refresh_crate))
    .route("/api/v1/crates/refresh_outdated", post(refresh_outdated_crates))
    .route("/api/v1/crates/refresh_all", post(refresh_all_crates))
    .with_state(app)
    ;

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal)
    .await?;

  Ok(())
}

async fn search_crates(State(app): State<App>, Json(search): Json<Search>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = app.data.write().await;
  let crates = app.crates.search(&mut data.crates, search).await.map_err(|_| F)?;
  Ok(Json(crates))
}
async fn get_crate(State(app): State<App>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = app.crates.get(&mut data.crates, &crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}

async fn follow_crate(State(app): State<App>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = app.crates.follow(&mut data.crates, &crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn unfollow_crate(State(app): State<App>, Path(crate_id): Path<String>) {
  let mut data = app.data.write().await;
  app.crates.unfollow(&mut data.crates, crate_id);
}

async fn refresh_crate(State(app): State<App>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = app.data.write().await;
  let krate = app.crates.refresh_one(&mut data.crates, crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn refresh_outdated_crates(State(app): State<App>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = app.data.write().await;
  let crates = app.crates.refresh_outdated(&mut data.crates).await.map_err(|_| F)?;
  Ok(Json(crates))
}
async fn refresh_all_crates(State(app): State<App>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = app.data.write().await;
  let crates = app.crates.refresh_all(&mut data.crates).await.map_err(|_| F)?;
  Ok(Json(crates))
}

// Error "utility"
struct F;
impl IntoResponse for F {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
  }
}
