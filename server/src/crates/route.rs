use axum::extract::{Path, Query, State};
use axum::Router;

use att_core::crates::{CrateError, CratesQuery, FullCrate};

use crate::crates::Crates;
use crate::users::AuthSession;
use crate::util::JsonResult;

pub fn router() -> Router<Crates> {
  use axum::routing::{get, post};
  Router::new()
    .route("/", get(search))
    .route("/:crate_id", get(find))
    .route("/:crate_id/follow", post(follow).delete(unfollow))
    .route("/:crate_id/refresh", post(refresh))
    .route("/refresh_followed", post(refresh_followed_crates))
}

async fn search(
  auth_session: AuthSession,
  State(state): State<Crates>,
  Query(query): Query<CratesQuery>
) -> JsonResult<Vec<FullCrate>, CrateError> {
  let user_id = auth_session.user.map(|u| u.id);
  let full_crates = state.search(query, user_id)
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(full_crates.into())
}

async fn find(State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<FullCrate, CrateError> {
  let full_crate = state.find(crate_id)
    .await
    .map_err(CrateError::from)?;
  Ok(full_crate.into())
}

async fn follow(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
  let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
  let krate = state.db_pool.query(move |db| db.follow(user_id, crate_id))
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(krate.into())
}

async fn unfollow(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
  let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
  state.db_pool.query(move |db| db.unfollow(user_id, crate_id))
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(().into())
}

async fn refresh(State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<FullCrate, CrateError> {
  let full_crate = state.refresh_one(crate_id).await
    .map_err(CrateError::from)?;
  Ok(full_crate.into())
}

async fn refresh_followed_crates(auth_session: AuthSession, State(state): State<Crates>) -> JsonResult<Vec<FullCrate>, CrateError> {
  let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
  let full_crates = state.refresh_followed(user_id).await
    .map_err(CrateError::from)?;
  Ok(full_crates.into())
}
