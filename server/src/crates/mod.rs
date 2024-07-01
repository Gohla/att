use std::error::Error;
use std::future::Future;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use thiserror::Error;
use tracing::instrument;

use att_core::crates::Crate;
use crates_io_client::CratesIoClient;

use crate::crates::crates_io_client::CratesIoClientError;
use crate::crates::crates_io_dump::{CratesIoDump, UpdateCratesIoDumpJob};
use crate::db::{DbError, DbPool};
use crate::users::User;

pub mod crates_io_client;
pub mod crates_io_dump;
pub mod route;

#[derive(Clone)]
pub struct Crates {
  db_pool: DbPool,
  crates_io_client: CratesIoClient,
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
    let crates = Self { db_pool, crates_io_client, crates_io_dump };
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
          .filter(name.ilike(format!("{}%", search_term)))
          .order(id)
          .load(conn)
      }).await??
    };
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn get(&self, crate_id: i32) -> Result<Option<Crate>, DbError> {
    let conn = self.db_pool.get().await?;
    let krate = conn.interact(move |conn| {
      use att_core::schema::crates;
      crates::table
        .find(crate_id)
        .first(conn)
        .optional()
    }).await??;
    Ok(krate)
  }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Insertable)]
#[diesel(table_name = att_core::schema::favorite_crates, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, crate_id), belongs_to(User), belongs_to(Crate))]
pub struct FavoriteCrate {
  pub user_id: i32,
  pub crate_id: i32,
}

impl Crates {
  #[instrument(skip(self))]
  pub async fn get_followed_crates(&self, user: User) -> Result<Vec<Crate>, DbError> {
    let conn = self.db_pool.get().await?;
    let crates = conn.interact(move |conn| {
      use att_core::schema::crates;
      FavoriteCrate::belonging_to(&user)
        .inner_join(crates::table)
        .select(Crate::as_select())
        .load(conn)
    }).await??;
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn follow(&self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    let conn = self.db_pool.get().await?;
    conn.interact(move |conn| {
      use att_core::schema::favorite_crates;
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
      diesel::delete(favorite_crates::table)
        .filter(favorite_crates::user_id.eq(user_id))
        .filter(favorite_crates::crate_id.eq(crate_id))
        .execute(conn)
    }).await??;
    Ok(())
  }
}


// crates.io API refresh

#[derive(Debug, Error)]
pub enum InternalError {
  #[error(transparent)]
  CratesIoClient(#[from] CratesIoClientError),
  #[error(transparent)]
  Database(DbError),
}
impl<E: Into<DbError>> From<E> for InternalError {
  fn from(value: E) -> Self { Self::Database(value.into()) }
}

#[derive(AsChangeset)]
#[diesel(table_name = att_core::schema::crates, check_for_backend(diesel::pg::Pg))]
pub struct UpdateCrate {
  pub updated_at: DateTime<Utc>,
}

impl Crates {
  #[instrument(skip(self), err)]
  pub async fn refresh_one(&self, crate_id: i32) -> Result<Option<Crate>, InternalError> {
    let conn = self.db_pool.get().await?;
    let krate = conn.interact(move |conn| {
      use att_core::schema::crates;
      crates::table
        .select(Crate::as_select())
        .find(crate_id)
        .first(conn)
        .optional()
    }).await??;
    if let Some(mut krate) = krate {
      // TODO: update more fields
      let response = self.crates_io_client.refresh(krate.name.clone()).await?;
      let updated_at = response.crate_data.updated_at;
      krate.updated_at = updated_at;
      conn.interact(move |conn| {
        use att_core::schema::crates;
        diesel::update(crates::table)
          .filter(crates::id.eq(crate_id))
          .set(UpdateCrate { updated_at })
          .execute(conn)
      }).await??;
      Ok(Some(krate))
    } else {
      Ok(None)
    }
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
