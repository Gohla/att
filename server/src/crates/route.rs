use axum::extract::{Path, Query, State};
use axum::Router;

use att_core::crates::{CrateError, CrateSearchQuery, FullCrate};

use crate::crates::Crates;
use crate::users::AuthSession;
use crate::util::JsonResult;

pub fn router() -> Router<Crates> {
  use axum::routing::{get, post};
  Router::new()
    .route("/", get(search_crates))
    .route("/:crate_id", get(find))
    .route("/:crate_id/follow", post(follow).delete(unfollow))
    .route("/:crate_id/refresh", post(refresh))
  // .route("/refresh_outdated", post(refresh_outdated_crates))
  // .route("/refresh_all", post(refresh_all_crates))
}

async fn search_crates(
  auth_session: AuthSession,
  State(state): State<Crates>,
  Query(search): Query<CrateSearchQuery>
) -> JsonResult<Vec<FullCrate>, CrateError> {
  let full_crates = match search {
    CrateSearchQuery { followed: true, .. } => {
      if let Some(user) = &auth_session.user {
        let user_id = user.id;
        state.db_pool.query(move |db| db.get_followed_crates(user_id))
          .await
          .map_err(|_| CrateError::Internal)?
      } else {
        Err(CrateError::NotLoggedIn)?
      }
    }
    CrateSearchQuery { search_term: Some(search_term), .. } => state.db_pool.perform(move |c| c.search(&search_term))
      .await
      .map_err(|_| CrateError::Internal)?,
    _ => Vec::default()
  };
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
//
// async fn refresh_outdated_crates(auth_session: AuthSession, State(state): State<Crates>) -> JsonResult<Vec<Crate>, CrateError> {
//   async move {
//     let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
//     let mut data = state.database.write().await;
//     let crates = state.crates.refresh_outdated(&mut data.crates, user_id).await
//       .map_err(CratesIoClientError::into_crate_error)?;
//     Ok(crates)
//   }.await.into()
// }
//
// async fn refresh_all_crates(auth_session: AuthSession, State(state): State<Crates>) -> JsonResult<Vec<Crate>, CrateError> {
//   async move {
//     let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
//     let mut data = state.database.write().await;
//     let crates = state.crates.refresh_all(&mut data.crates, user_id).await
//       .map_err(CratesIoClientError::into_crate_error)?;
//     Ok(crates)
//   }.await.into()
// }
