use axum::extract::{Path, Query, State};
use axum::Router;

use att_core::crates::{Crate, CrateError, CrateSearchQuery};

use crate::crates::Crates;
use crate::users::AuthSession;
use crate::util::JsonResult;

pub fn router() -> Router<Crates> {
  use axum::routing::{get, post};
  Router::new()
    .route("/", get(search_crates))
    .route("/:crate_id", get(get_crate))
    .route("/:crate_id/follow", post(follow_crate).delete(unfollow_crate))
  // .route("/:crate_id/refresh", post(refresh_crate))
  // .route("/refresh_outdated", post(refresh_outdated_crates))
  // .route("/refresh_all", post(refresh_all_crates))
}

async fn search_crates(
  auth_session: AuthSession,
  State(state): State<Crates>,
  Query(search): Query<CrateSearchQuery>
) -> JsonResult<Vec<Crate>, CrateError> {
  let crates = match search {
    CrateSearchQuery { followed: true, .. } => {
      if let Some(user) = &auth_session.user {
        state.get_followed_crates(user.clone())
          .await
          .map_err(|_| CrateError::Internal)?
      } else {
        Err(CrateError::NotLoggedIn)?
      }
    }
    CrateSearchQuery { search_term: Some(search_term), .. } => state
      .search(search_term)
      .await
      .map_err(|_| CrateError::Internal)?,
    _ => Vec::default()
  };
  Ok(crates.into())
}

async fn get_crate(State(state): State<Crates>, Path(crate_name): Path<String>) -> JsonResult<Option<Crate>, CrateError> {
  let krate = state.get(crate_name)
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(krate.into())
}

async fn follow_crate(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
  let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
  let krate = state.follow(user_id, crate_id)
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(krate.into())
}

async fn unfollow_crate(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
  let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
  state.unfollow(user_id, crate_id)
    .await
    .map_err(|_| CrateError::Internal)?;
  Ok(().into())
}

// async fn refresh_crate(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<String>) -> JsonResult<Crate, CrateError> {
//   async move {
//     let _ = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id();
//     let mut data = state.database.write().await;
//     let krate = state.crates.refresh_one(&mut data.crates, crate_id).await
//       .map_err(CratesIoClientError::into_crate_error)?;
//     Ok(krate)
//   }.await.into()
// }
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
