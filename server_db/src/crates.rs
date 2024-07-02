use chrono::{DateTime, Utc};
use diesel::{copy_from, delete, insert_into};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::{debug, instrument};

use att_core::crates::{Crate, CrateDefaultVersion, CrateDownloads, CrateVersion};
use att_core::schema::{crate_default_versions, crate_downloads, crate_versions, crates, favorite_crates, import_crates_metadata};

use crate::{DbError, DbPool};
use crate::users::User;

#[derive(Clone)]
pub struct CratesDb {
  pool: DbPool,
}

impl CratesDb {
  pub fn new(pool: DbPool) -> Self {
    Self { pool }
  }
}


// Select crates

impl CratesDb {
  #[instrument(skip(self), err)]
  pub async fn find(&self, crate_id: i32) -> Result<Option<Crate>, DbError> {
    let krate = self.pool.connect_and_query(move |conn| {
      crates::table
        .find(crate_id)
        .first(conn)
        .optional()
    }).await?;
    Ok(krate)
  }

  #[instrument(skip(self), err)]
  pub async fn search(&self, search_term: String) -> Result<Vec<Crate>, DbError> {
    let crates = self.pool.connect_and_query(move |conn| {
      crates::table
        .filter(crates::name.ilike(format!("{}%", search_term)))
        .order(crates::id)
        .load(conn)
    }).await?;
    Ok(crates)
  }
}


// Update crates

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crates, check_for_backend(Pg))]
pub struct UpdateCrate {
  pub updated_at: DateTime<Utc>,
}

impl CratesDb {
  #[instrument(skip(self), err)]
  pub async fn update(&self, crate_id: i32, update_crate: UpdateCrate) -> Result<(), DbError> {
    self.pool.connect_and_query(move |conn| {
      diesel::update(crates::table)
        .filter(crates::id.eq(crate_id))
        .set(update_crate)
        .execute(conn)
    }).await?;
    Ok(())
  }
}


// Import crates


pub struct ImportCrates {
  pub crates: Vec<Crate>,
  pub downloads: Vec<CrateDownloads>,
  pub versions: Vec<CrateVersion>,
  pub default_versions: Vec<CrateDefaultVersion>,
}
impl Default for ImportCrates {
  fn default() -> Self {
    const EXPECTED_CRATE_COUNT: usize = 1024 * 512;
    Self {
      crates: Vec::with_capacity(EXPECTED_CRATE_COUNT),
      downloads: Vec::with_capacity(EXPECTED_CRATE_COUNT),
      versions: Vec::with_capacity(EXPECTED_CRATE_COUNT * 2),
      default_versions: Vec::with_capacity(EXPECTED_CRATE_COUNT),
    }
  }
}

impl CratesDb {
  pub async fn import(&self, import_crates: ImportCrates) -> Result<usize, DbError> {
    let inserted_rows = self.pool.connect_and_query(move |conn| {
      let mut inserted_rows: usize = 0;

      debug!("Deleting table `crate_default_versions`");
      delete(crate_default_versions::table).execute(conn)?;
      debug!("Deleting table `crate_versions`");
      delete(crate_versions::table).execute(conn)?;
      debug!("Deleting table `crate_downloads`");
      delete(crate_downloads::table).execute(conn)?;
      debug!("Deleting table `crates`");
      delete(crates::table).execute(conn)?;

      debug!("Copying {} crates into `crates`", import_crates.crates.len());
      inserted_rows += copy_from(crates::table)
        .from_insertable(import_crates.crates)
        .execute(conn)?;

      debug!("Copying {} downloads into `crate_downloads`", import_crates.downloads.len());
      inserted_rows += copy_from(crate_downloads::table)
        .from_insertable(import_crates.downloads)
        .execute(conn)?;

      debug!("Copying {} versions into `crate_versions`", import_crates.versions.len());
      inserted_rows += copy_from(crate_versions::table)
        .from_insertable(import_crates.versions)
        .execute(conn)?;

      debug!("Copying {} default versions into `crate_default_versions`", import_crates.default_versions.len());
      inserted_rows += copy_from(crate_default_versions::table)
        .from_insertable(import_crates.default_versions)
        .execute(conn)?;

      debug!("Inserting entry into `import_crates_metadata`");
      inserted_rows += insert_into(import_crates_metadata::table)
        .values(import_crates_metadata::imported_at.eq(Utc::now()))
        .execute(conn)?;

      Ok(inserted_rows)
    }).await?;
    Ok(inserted_rows)
  }

  pub async fn get_last_imported_at(&self) -> Result<Option<DateTime<Utc>>, DbError> {
    let last_imported_at = self.pool.connect_and_query(|conn| {
      import_crates_metadata::table
        .select(import_crates_metadata::imported_at)
        .order(import_crates_metadata::id.desc())
        .first(conn)
        .optional()
    }).await?;
    Ok(last_imported_at)
  }
}


// Query favorite crates

#[derive(Debug, Identifiable, Selectable, Queryable, Associations, Insertable)]
#[diesel(table_name = favorite_crates, check_for_backend(Pg))]
#[diesel(primary_key(user_id, crate_id), belongs_to(User), belongs_to(Crate))]
pub struct FavoriteCrate {
  pub user_id: i32,
  pub crate_id: i32,
}

impl CratesDb {
  #[instrument(skip(self))]
  pub async fn get_followed(&self, user: User) -> Result<Vec<Crate>, DbError> {
    let crates = self.pool.connect_and_query(move |conn| {
      FavoriteCrate::belonging_to(&user)
        .inner_join(crates::table)
        .select(Crate::as_select())
        .load(conn)
    }).await?;
    Ok(crates)
  }

  #[instrument(skip(self), err)]
  pub async fn follow(&self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    self.pool.connect_and_query(move |conn| {
      insert_into(favorite_crates::table)
        .values(&FavoriteCrate { crate_id, user_id })
        .execute(conn)
    }).await?;
    Ok(())
  }

  #[instrument(skip(self), err)]
  pub async fn unfollow(&self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    self.pool.connect_and_query(move |conn| {
      delete(favorite_crates::table)
        .filter(favorite_crates::user_id.eq(user_id))
        .filter(favorite_crates::crate_id.eq(crate_id))
        .execute(conn)
    }).await?;
    Ok(())
  }
}
