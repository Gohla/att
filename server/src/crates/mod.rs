use std::error::Error;
use std::future::Future;
use std::path::PathBuf;

use axum::extract::{Path, Query, State};
use axum::Router;
use diesel::prelude::*;
use tracing::instrument;

use att_core::crates::{Crate, CrateError, CrateSearchQuery};
use crates_io_client::CratesIoClient;

use crate::crates::crates_io_dump::{CratesIoDump, UpdateCratesIoDumpJob};
use crate::db::{DbError, DbPool};
use crate::users::{AuthSession, User};
use crate::util::JsonResult;

pub mod crates_io_client;
pub mod crates_io_dump;

#[derive(Clone)]
pub struct Crates {
  db_pool: DbPool,
  _crates_io_client: CratesIoClient,
  crates_io_dump: CratesIoDump,
}

impl Crates {
  pub fn new(
    db_pool: DbPool,
    crates_io_user_agent: &str,
    crates_io_db_dump_file: PathBuf
  ) -> Result<(Self, impl Future<Output=()>), Box<dyn Error>> {
    let (crates_io_client, task) = CratesIoClient::new(crates_io_user_agent)?;
    let crates_io_dump = CratesIoDump::new(crates_io_db_dump_file, db_pool.clone());
    let crates = Self { db_pool, _crates_io_client: crates_io_client, crates_io_dump };
    Ok((crates, task))
  }

  pub fn create_update_crates_io_dump_job(&self) -> UpdateCratesIoDumpJob {
    UpdateCratesIoDumpJob::new(self.crates_io_dump.clone())
  }
}

impl Crates {
  #[instrument(skip(self), err)]
  pub async fn search(&self, search_term: String) -> Result<Vec<Crate>, DbError> {
    let crates = {
      let conn = self.db_pool.get().await?;
      conn.interact(move |conn| {
        use att_core::schema::crates::dsl::*;
        crates
          .select(Crate::as_select())
          .filter(name.ilike(format!("{}%", search_term)))
          .order(id)
          .load(conn)
      }).await??
    };
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn get(&self, crate_name: String) -> Result<Option<Crate>, DbError> {
    let conn = self.db_pool.get().await?;
    let krate = conn.interact(|conn| {
      use att_core::schema::crates::dsl::*;
      crates
        .select(Crate::as_select())
        .filter(name.eq(crate_name))
        .first(conn)
        .optional()
    }).await??;
    Ok(krate)
  }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Insertable, Debug)]
#[diesel(table_name = att_core::schema::favorite_crates, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, crate_id), belongs_to(User), belongs_to(Crate))]
pub struct FavoriteCrate {
  pub user_id: i32,
  pub crate_id: i32,
}

impl Crates {
  #[instrument(skip(self))]
  pub async fn get_followed_crates(&self, user: User) -> Result<Vec<Crate>, DbError> {
    let crates = {
      use att_core::schema::crates;
      let conn = self.db_pool.get().await?;
      conn.interact(move |conn| {
        FavoriteCrate::belonging_to(&user)
          .inner_join(crates::table)
          .select(Crate::as_select())
          .load(conn)
      }).await??
    };
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn follow(&self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    use att_core::schema::favorite_crates;
    let conn = self.db_pool.get().await?;
    conn.interact(move |conn| {
      diesel::insert_into(favorite_crates::table)
        .values(&FavoriteCrate { crate_id, user_id })
        .execute(conn)
    }).await??;
    Ok(())
  }

  #[instrument(skip(self), err)]
  pub async fn unfollow(&self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    let conn = self.db_pool.get().await?;
    conn.interact(move |conn| {
      use att_core::schema::favorite_crates;
      diesel::delete(favorite_crates::table
        .filter(favorite_crates::user_id.eq(user_id))
        .filter(favorite_crates::crate_id.eq(crate_id))
      )
        .execute(conn)
    }).await??;
    Ok(())
  }
}
// impl Crates {
//   #[instrument(skip(self), err)]
//   pub async fn refresh_one(&self, crate_id: String) -> Result<Crate, CratesIoClientError> {
//     self.ensure_refreshed(&mut data.id_to_crate, &crate_id, Utc::now(), |_, _| true).await
//   }
//
//   #[instrument(skip(self), err)]
//   pub async fn refresh_outdated(&self, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
//     self.refresh_multiple(data, user_id, Utc::now(), refresh_hourly).await
//   }
//
//   #[instrument(skip(self), err)]
//   pub async fn refresh_all(&self, user_id: u64) -> Result<Vec<Crate>, CratesIoClientError> {
//     self.refresh_multiple(data, user_id, Utc::now(), |_, _| true).await
//   }
//
//
//   #[instrument(skip_all, err)]
//   async fn refresh_for_all_users(
//     &self,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Vec<Crate>, CratesIoClientError> {
//     // TODO: remove data from unfollowed crates? Probably best done in a separate step and done in a job.
//     let mut refreshed = Vec::new();
//     // Refresh outdated cached crate data.
//     for (krate, last_refreshed) in data.id_to_crate.values_mut() {
//       let crate_id = &krate.name;
//       if should_refresh(&now, last_refreshed) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         *krate = response.crate_data.into();
//         *last_refreshed = now;
//         refreshed.push(krate.clone());
//       }
//     }
//     // Refresh missing cached crate data.
//     for crate_id in data.followed_crate_ids.values().flatten() {
//       if !data.id_to_crate.contains_key(crate_id) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         let krate: Crate = response.crate_data.into();
//         data.id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
//         refreshed.push(krate);
//       }
//     }
//     Ok(refreshed)
//   }
//
//   #[instrument(skip_all, err)]
//   async fn refresh_multiple(
//     &self,
//     user_id: u64,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Vec<Crate>, CratesIoClientError> {
//     let mut refreshed = Vec::new();
//     if let Some(followed_crate_ids) = data.followed_crate_ids.get(&user_id) {
//       for crate_id in followed_crate_ids {
//         let krate = self.ensure_refreshed(&mut data.id_to_crate, crate_id, now, &should_refresh).await?;
//         refreshed.push(krate);
//       }
//     }
//     Ok(refreshed)
//   }
//
//   async fn ensure_refreshed(
//     &self,
//     id_to_crate: &mut BTreeMap<String, (Crate, DateTime<Utc>)>,
//     crate_id: &String,
//     now: DateTime<Utc>,
//     should_refresh: impl Fn(&DateTime<Utc>, &DateTime<Utc>) -> bool
//   ) -> Result<Crate, CratesIoClientError> {
//     let krate = if let Some((krate, last_refreshed)) = id_to_crate.get_mut(crate_id) {
//       if should_refresh(&now, last_refreshed) {
//         let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//         *krate = response.crate_data.into();
//         *last_refreshed = now;
//       }
//       krate.clone()
//     } else {
//       let response = self.crates_io_client.refresh(crate_id.clone()).await?;
//       let krate: Crate = response.crate_data.into();
//       id_to_crate.insert(crate_id.clone(), (krate.clone(), now));
//       krate
//     }; // Note: can't use entry API due to async.
//     Ok(krate)
//   }
// }
//
// fn refresh_hourly(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>) -> bool {
//   now.signed_duration_since(last_refresh) > Duration::hours(1)
// }


// Routing

pub fn router() -> Router<Crates> {
  use axum::routing::{get, post};
  return Router::new()
    .route("/", get(search_crates))
    .route("/:crate_id", get(get_crate))
    .route("/:crate_id/follow", post(follow_crate).delete(unfollow_crate))
  // .route("/:crate_id/refresh", post(refresh_crate))
  // .route("/refresh_outdated", post(refresh_outdated_crates))
  // .route("/refresh_all", post(refresh_all_crates))
  ;

  async fn search_crates(
    auth_session: AuthSession,
    State(state): State<Crates>,
    Query(search): Query<CrateSearchQuery>
  ) -> JsonResult<Vec<Crate>, CrateError> {
    async move {
      let crates = match search {
        CrateSearchQuery { followed: true, .. } => {
          if let Some(user) = &auth_session.user {
            state.get_followed_crates(user.clone())
              .await
              .map_err(|_| CrateError::Internal)?
          } else {
            return Err(CrateError::NotLoggedIn)
          }
        }
        CrateSearchQuery { search_term: Some(search_term), .. } => state
          .search(search_term)
          .await
          .map_err(|_| CrateError::Internal)?,
        _ => Vec::default()
      };
      Ok(crates)
    }.await.into()
  }

  async fn get_crate(State(state): State<Crates>, Path(crate_name): Path<String>) -> JsonResult<Option<Crate>, CrateError> {
    async move {
      let krate = state.get(crate_name)
        .await
        .map_err(|_| CrateError::Internal)?;
      Ok(krate)
    }.await.into()
  }

  async fn follow_crate(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
    async move {
      let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
      let krate = state.follow(user_id, crate_id)
        .await
        .map_err(|_| CrateError::Internal)?;
      Ok(krate)
    }.await.into()
  }

  async fn unfollow_crate(auth_session: AuthSession, State(state): State<Crates>, Path(crate_id): Path<i32>) -> JsonResult<(), CrateError> {
    async move {
      let user_id = auth_session.user.ok_or(CrateError::NotLoggedIn)?.id;
      state.unfollow(user_id, crate_id)
        .await
        .map_err(|_| CrateError::Internal)?;
      Ok(())
    }.await.into()
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
}
