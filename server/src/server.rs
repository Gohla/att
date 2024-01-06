use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_login::AuthManagerLayerBuilder;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions::cookie::time::Duration;

use att_core::{Crate, Search};

use crate::auth;
use crate::auth::Authenticator;
use crate::data::Database;
use crate::krate::Crates;

#[derive(Clone)]
pub struct Server {
  database: Database,
  crates: Crates,
}

impl Server {
  pub fn new(database: Database, crates: Crates) -> Self {
    Self { database, crates }
  }

  pub async fn run(self, shutdown_signal: impl Future<Output=()> + Send + 'static) -> Result<(), Box<dyn Error>> {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
      .with_expiry(Expiry::OnInactivity(Duration::days(30)))
      ;

    let authenticator = Authenticator::new(self.database.clone());
    let auth_layer = AuthManagerLayerBuilder::new(authenticator, session_layer.clone())
      .build();


    use axum::routing::{get, post};
    let api_routes = Router::new()
      .route("/crates", get(search_crates))
      .route("/crates/:crate_id", get(get_crate))
      .route("/crates/:crate_id/follow", post(follow_crate).delete(unfollow_crate))
      .route("/crates/:crate_id/refresh", post(refresh_crate))
      .route("/crates/refresh_outdated", post(refresh_outdated_crates))
      .route("/crates/refresh_all", post(refresh_all_crates))
      .nest("/users", auth::router().with_state(()))
      ;

    let router = Router::new()
      .nest("/api/v1/", api_routes)
      .with_state(self)
      .layer(session_layer)
      .layer(auth_layer)
      ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
      .with_graceful_shutdown(shutdown_signal)
      .await?;

    Ok(())
  }
}

async fn search_crates(State(server): State<Server>, Json(search): Json<Search>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = server.database.write().await;
  let crates = server.crates.search(&mut data.crates, search).await.map_err(|_| F)?;
  Ok(Json(crates))
}
async fn get_crate(State(server): State<Server>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = server.database.write().await;
  let krate = server.crates.get(&mut data.crates, &crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}

async fn follow_crate(State(server): State<Server>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = server.database.write().await;
  let krate = server.crates.follow(&mut data.crates, &crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn unfollow_crate(State(server): State<Server>, Path(crate_id): Path<String>) {
  let mut data = server.database.write().await;
  server.crates.unfollow(&mut data.crates, crate_id);
}

async fn refresh_crate(State(server): State<Server>, Path(crate_id): Path<String>) -> Result<Json<Crate>, F> {
  let mut data = server.database.write().await;
  let krate = server.crates.refresh_one(&mut data.crates, crate_id).await.map_err(|_| F)?;
  Ok(Json(krate))
}
async fn refresh_outdated_crates(State(server): State<Server>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = server.database.write().await;
  let crates = server.crates.refresh_outdated(&mut data.crates).await.map_err(|_| F)?;
  Ok(Json(crates))
}
async fn refresh_all_crates(State(server): State<Server>) -> Result<Json<Vec<Crate>>, F> {
  let mut data = server.database.write().await;
  let crates = server.crates.refresh_all(&mut data.crates).await.map_err(|_| F)?;
  Ok(Json(crates))
}

// Error "utility"
struct F;
impl IntoResponse for F {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
  }
}
