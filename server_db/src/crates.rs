use chrono::{DateTime, Utc};
use diesel::{copy_from, delete, insert_into};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::{debug, instrument};

use att_core::crates::{Crate, CrateVersion, FullCrate};
use att_core::schema::{crate_versions, crates, favorite_crates, import_crates_metadata};

use crate::{DbConn, DbError};
use crate::users::User;

#[derive(Copy, Clone)]
pub struct CratesDb;


// Select crates

impl DbConn<'_, CratesDb> {
  #[instrument(skip(self), err)]
  pub fn find(&mut self, crate_id: i32) -> Result<Option<FullCrate>, DbError> {
    let full_crate = crates::table
      .find(crate_id)
      .inner_join(crate_versions::table.on(crate_versions::id.eq(crates::default_version_id)))
      .select(FullCrate::as_select())
      .first(self.conn)
      .optional()?;
    Ok(full_crate)
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
  pub fn search(&mut self, search_term: &str) -> Result<Vec<FullCrate>, DbError> {
    let full_crates = crates::table
      .filter(crates::name.ilike(format!("{}%", search_term)))
      .order(crates::id)
      .inner_join(crate_versions::table.on(crate_versions::id.eq(crates::default_version_id)))
      .select(FullCrate::as_select())
      .load(self.conn)?;
    Ok(full_crates)
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

  pub downloads: Option<i64>,
}

#[derive(Debug, Identifiable, AsChangeset)]
#[diesel(table_name = crate_versions, check_for_backend(Pg))]
pub struct UpdateVersion {
  pub id: i32,
  pub crate_id: i32,
  pub number: String,
}

impl DbConn<'_, CratesDb> {
  #[instrument(skip(self), err)]
  pub fn update_crate(&mut self, update: UpdateCrate) -> Result<Option<Crate>, DbError> {
    let krate = update.save_changes::<Crate>(self.conn).optional()?;
    Ok(krate)
  }

  #[instrument(skip(self), err)]
  pub fn update_crate_version(&mut self, update: UpdateVersion) -> Result<Option<CrateVersion>, DbError> {
    let version = update.save_changes::<CrateVersion>(self.conn).optional()?;
    Ok(version)
  }
}


// Import crates

pub struct ImportCrates {
  pub crates: Vec<Crate>,
  pub versions: Vec<CrateVersion>,
}
impl ImportCrates {
  pub fn with_expected_crate_count(count: usize) -> Self {
    Self {
      crates: Vec::with_capacity(count),
      versions: Vec::with_capacity(count * 2),
    }
  }
}

impl DbConn<'_, CratesDb> {
  #[instrument(skip_all, err)]
  pub fn import(&mut self, import_crates: ImportCrates) -> Result<usize, DbError> {
    let inserted_rows = self.conn.transaction(|conn| {
      let mut inserted_rows: usize = 0;

      debug!("Deleting table `crate_versions`");
      delete(crate_versions::table).execute(conn)?;
      debug!("Deleting table `crates`");
      delete(crates::table).execute(conn)?;

      debug!("Copying {} crates into `crates`", import_crates.crates.len());
      inserted_rows += copy_from(crates::table)
        .from_insertable(import_crates.crates)
        .execute(conn)?;

      debug!("Copying {} versions into `crate_versions`", import_crates.versions.len());
      inserted_rows += copy_from(crate_versions::table)
        .from_insertable(import_crates.versions)
        .execute(conn)?;

      debug!("Inserting entry into `import_crates_metadata`");
      inserted_rows += insert_into(import_crates_metadata::table)
        .values(import_crates_metadata::imported_at.eq(Utc::now()))
        .execute(conn)?;

      Ok::<_, DbError>(inserted_rows)
    })?;

    Ok(inserted_rows)
  }

  #[instrument(skip(self), err)]
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
  #[instrument(skip(self), err)]
  pub fn get_followed_crates(&mut self, user_id: i32) -> Result<Vec<FullCrate>, DbError> {
    let full_crates = favorite_crates::table
      .filter(favorite_crates::user_id.eq(user_id))
      .inner_join(crates::table)
      .inner_join(crate_versions::table.on(crate_versions::id.eq(crates::default_version_id)))
      .select(FullCrate::as_select())
      .load::<FullCrate>(self.conn)?;
    Ok(full_crates)
  }

  #[instrument(skip(self), err)]
  pub fn get_followed_crate_ids(&mut self, user_id: i32) -> Result<Vec<i32>, DbError> {
    let crates_ids = favorite_crates::table
      .filter(favorite_crates::user_id.eq(user_id))
      .inner_join(crates::table)
      .select(crates::id)
      .load(self.conn)?;
    Ok(crates_ids)
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
