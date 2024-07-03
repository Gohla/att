use chrono::{DateTime, Utc};
use diesel::{copy_from, delete, insert_into};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::{debug, instrument};

use att_core::crates::{Crate, CrateDefaultVersion, CrateDownloads, CrateVersion};
use att_core::schema::{crate_default_versions, crate_downloads, crate_versions, crates, favorite_crates, import_crates_metadata};

use crate::{DbConn, DbError};
use crate::users::User;

#[derive(Copy, Clone)]
pub struct CratesDb;


// Select crates

impl DbConn<'_, CratesDb> {
  #[instrument(skip(self), err)]
  pub fn find(&mut self, crate_id: i32) -> Result<Option<Crate>, DbError> {
    let krate = crates::table
      .find(crate_id)
      .first(self.conn)
      .optional()?;
    Ok(krate)
  }

  #[instrument(skip(self), err)]
  pub fn find_name(&mut self, crate_id: i32) -> Result<Option<String>, DbError> {
    let crate_name = crates::table
      .find(crate_id)
      .select(crates::name)
      .first(self.conn)
      .optional()?;
    Ok(crate_name)
  }

  #[instrument(skip(self), err)]
  pub fn search(&mut self, search_term: String) -> Result<Vec<Crate>, DbError> {
    let crates = crates::table
      .filter(crates::name.ilike(format!("{}%", search_term)))
      .order(crates::id)
      .load(self.conn)?;
    Ok(crates)
  }
}


// Update crates

#[derive(Default, Debug, Identifiable, AsChangeset)]
#[diesel(table_name = crates, check_for_backend(Pg))]
pub struct UpdateCrate {
  pub id: i32,
  pub updated_at: Option<DateTime<Utc>>,
  pub description: Option<String>,
  pub homepage: Option<Option<String>>,
  pub readme: Option<Option<String>>,
  pub repository: Option<Option<String>>,
}

#[derive(Debug, Identifiable, AsChangeset)]
#[diesel(table_name = crate_downloads, primary_key(crate_id), check_for_backend(Pg))]
pub struct UpdateDownloads {
  pub crate_id: i32,
  pub downloads: i64,
}

impl DbConn<'_, CratesDb> {
  #[instrument(skip(self), err)]
  pub fn update_crate(&mut self, update: UpdateCrate) -> Result<Option<Crate>, DbError> {
    let krate = update.save_changes::<Crate>(self.conn).optional()?;
    Ok(krate)
  }

  #[instrument(skip(self), err)]
  pub fn update_crate_downloads(&mut self, update: UpdateDownloads) -> Result<Option<CrateDownloads>, DbError> {
    let crate_downloads = update.save_changes::<CrateDownloads>(self.conn).optional()?;
    Ok(crate_downloads)
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

impl DbConn<'_, CratesDb> {
  pub fn import(&mut self, import_crates: ImportCrates) -> Result<usize, DbError> {
    let inserted_rows = self.conn.transaction(|conn| {
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

      Ok::<_, DbError>(inserted_rows)
    })?;

    Ok(inserted_rows)
  }

  pub fn get_last_imported_at(&mut self) -> Result<Option<DateTime<Utc>>, DbError> {
    let last_imported_at = import_crates_metadata::table
      .select(import_crates_metadata::imported_at)
      .order(import_crates_metadata::id.desc())
      .first(self.conn)
      .optional()?;
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

impl DbConn<'_, CratesDb> {
  #[instrument(skip(self))]
  pub fn get_followed_crates(&mut self, user: User) -> Result<Vec<Crate>, DbError> {
    let crates = FavoriteCrate::belonging_to(&user)
      .inner_join(crates::table)
      .select(Crate::as_select())
      .load(self.conn)?;
    Ok(crates)
  }

  #[instrument(skip(self))]
  pub fn get_followed_crate_ids(&mut self, user: User) -> Result<Vec<i32>, DbError> {
    let crate_ids = FavoriteCrate::belonging_to(&user)
      .select(favorite_crates::crate_id)
      .load(self.conn)?;
    Ok(crate_ids)
  }

  #[instrument(skip(self), err)]
  pub fn follow(&mut self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    insert_into(favorite_crates::table)
      .values(&FavoriteCrate { crate_id, user_id })
      .execute(self.conn)?;
    Ok(())
  }

  #[instrument(skip(self), err)]
  pub fn unfollow(&mut self, user_id: i32, crate_id: i32) -> Result<(), DbError> {
    delete(favorite_crates::table)
      .filter(favorite_crates::user_id.eq(user_id))
      .filter(favorite_crates::crate_id.eq(crate_id))
      .execute(self.conn)?;
    Ok(())
  }
}
