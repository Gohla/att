use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::{Json, Router, routing::get};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use tokio::sync::RwLock;

use att_core::Crate;

use crate::crates_client::CratesClient;
use crate::data::{Cache, Data};

#[derive(Clone)]
pub struct AppState {
  pub data: Arc<RwLock<Data>>,
  pub cache: Arc<RwLock<Cache>>,
  pub crates_client: CratesClient,
}

pub async fn run(state: AppState, shutdown_signal: impl Future<Output=()>) -> Result<(), Box<dyn Error>> {
  let app = Router::new()
    .route("/crate/blessed", get(crate_blessed))
    .route("/crate/search/:term", get(crate_search))
    .route("/crate/refresh/outdated", get(crate_refresh_outdated))
    .route("/crate/refresh/all", get(crate_refresh_all))
    .route("/crate/refresh/:id", get(crate_refresh))
    .with_state(state)
    ;

  let addr = SocketAddr::from(([127, 0, 0, 1], 1337));
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .with_graceful_shutdown(shutdown_signal)
    .await?;

  Ok(())
}

async fn crate_blessed(State(state): State<AppState>) -> Json<Vec<Crate>> {
  let data = state.data.read().await;
  let cache = state.cache.read().await;

  let mut crates = Vec::with_capacity(data.blessed_crate_ids.len());
  for id in &data.blessed_crate_ids {
    if let Some((c, _)) = cache.crate_data.get(id) {
      crates.push(Crate {
        id: id.clone(),
        downloads: c.downloads,
        updated_at: c.updated_at,
        max_version: c.max_version.clone(),
      });
    } else {
      crates.push(Crate { id: id.clone(), ..Crate::default() });
    }
  }

  Json(crates)
}

async fn crate_search(State(state): State<AppState>, Path(term): Path<String>) -> Result<Json<Vec<Crate>>, F> {
  let response = state.crates_client.search(Instant::now(), term)
    .await
    .map_err(|_| F)?
    .map_err(|_| F)?;
  let crates: Vec<_> = response.crates.into_iter().map(|c| Crate {
    id: c.id,
    downloads: c.downloads,
    updated_at: c.updated_at,
    max_version: c.max_version,
  }).collect();
  Ok(Json(crates))
}

async fn crate_refresh_outdated(State(state): State<AppState>) -> Result<(), F> {
  refresh_outdated_cached_crate_data(&state).await
    .map_err(|_| F)
}

async fn crate_refresh_all(State(state): State<AppState>) -> Result<(), F> {
  refresh_all_cached_crate_data(&state).await
    .map_err(|_| F)
}

async fn crate_refresh(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Crate>, F> {
  let response = state.crates_client.clone().refresh(id.clone())
    .await
    .map_err(|_| F)?
    .map_err(|_| F)?;

  let krate = Crate {
    id: id.clone(),
    downloads: response.crate_data.downloads,
    updated_at: response.crate_data.updated_at,
    max_version: response.crate_data.max_version.clone(),
  };

  let mut cache = state.cache.write().await;
  cache.crate_data.insert(id, (response.crate_data, Utc::now()));

  Ok(Json(krate))
}


// Crate refresh utilities
async fn refresh_outdated_cached_crate_data(state: &AppState) -> Result<(), Box<dyn Error>> {
  let now = Utc::now();
  refresh_cached_crates_data(
    state,
    now,
    |last_refresh| now.signed_duration_since(last_refresh) > Duration::hours(1)
  ).await
}

async fn refresh_all_cached_crate_data(state: &AppState) -> Result<(), Box<dyn Error>> {
  refresh_cached_crates_data(state, Utc::now(), |_| true).await
}

async fn refresh_cached_crates_data(
  state: &AppState,
  now: DateTime<Utc>,
  should_refresh: impl Fn(&DateTime<Utc>) -> bool
) -> Result<(), Box<dyn Error>> {
  let data = state.data.read().await;
  let mut cache = state.cache.write().await;
  // Refresh outdated cached crate data.
  for (krate, last_refreshed) in cache.crate_data.values_mut() {
    let id = &krate.id;
    if data.blessed_crate_ids.contains(id) {
      if should_refresh(last_refreshed) {
        let response = state.crates_client.clone().refresh(id.clone()).await??;
        *krate = response.crate_data;
        *last_refreshed = now;
      }
    }
  }
  // Refresh missing cached crate data.
  for id in &data.blessed_crate_ids {
    if !cache.crate_data.contains_key(id) {
      let response = state.crates_client.clone().refresh(id.clone()).await??;
      cache.crate_data.insert(id.clone(), (response.crate_data, now));
    }
  }
  Ok(())
}


// Error "utility"
struct F;
impl IntoResponse for F {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
  }
}
