use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "diesel")]
use {crate::schema, diesel::{pg::Pg, prelude::*}};

use crate::table::{AsTableRow, Column};

/// A Rust crate.
#[cfg_attr(feature = "diesel",
  derive(Queryable, Selectable, Identifiable, Associations, AsChangeset, Insertable),
  diesel(
    table_name = schema::crates, belongs_to(CrateVersion, foreign_key = default_version_id), treat_none_as_default_value = false, check_for_backend(Pg)
  ),
)]
#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Crate {
  pub id: i32,
  pub name: String,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub description: String,
  pub homepage: Option<String>,
  pub readme: Option<String>,
  pub repository: Option<String>,

  pub downloads: i64,

  pub default_version_id: i32,
}

/// A version of a crate.
#[cfg_attr(feature = "diesel",
  derive(Queryable, Selectable, Identifiable, Associations, AsChangeset, Insertable),
  diesel(
    table_name = schema::crate_versions, treat_none_as_default_value = false, check_for_backend(Pg),
    belongs_to(Crate)
  ),
)]
#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CrateVersion {
  pub id: i32,
  pub crate_id: i32,
  pub number: String,
}

/// A crate along with its associated data.
#[cfg_attr(feature = "diesel", derive(Selectable, Queryable), diesel(check_for_backend(Pg)))]
#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct FullCrate {
  #[cfg_attr(feature = "diesel", diesel(embed))]
  pub krate: Crate,
  #[cfg_attr(feature = "diesel", diesel(embed))]
  pub default_version: CrateVersion,
}

impl AsTableRow for FullCrate {
  const COLUMNS: &'static [Column] = &[
    Column::with_default_alignment("Id", 0.5),
    Column::with_default_alignment("Name", 1.0),
    Column::with_default_alignment("Updated At", 1.0),
    Column::with_default_alignment("Latest Version", 1.0),
    Column::with_default_alignment("Downloads", 1.0),
    Column::with_default_alignment("Description", 2.0),
  ];

  fn cell(&self, column_index: u8) -> Option<Cow<str>> {
    let str = match column_index {
      0 => Cow::from(format!("{}", self.krate.id)),
      1 => Cow::from(&self.krate.name),
      2 => Cow::from(self.krate.updated_at.format("%Y-%m-%d").to_string()),
      3 => Cow::from(&self.default_version.number),
      4 => Cow::from(format!("{}", self.krate.downloads)),
      5 => Cow::from(&self.krate.description),
      _ => return None,
    };
    Some(str)
  }
}


#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CrateSearchQuery {
  pub search_term: Option<String>,
  pub followed: bool,
}

impl CrateSearchQuery {
  #[inline]
  pub fn from_term(search_term: String) -> Self { Self { search_term: Some(search_term), ..Self::default() } }
  #[inline]
  pub fn from_followed() -> Self { Self { followed: true, ..Self::default() } }

  #[inline]
  pub fn search_term(&self) -> &str { self.search_term.as_deref().unwrap_or_default() }

  #[inline]
  pub fn is_empty(&self) -> bool {
    let Some(search_term) = &self.search_term else {
      return false;
    };
    if !search_term.is_empty() {
      return false;
    }
    !self.followed
  }
}

impl From<String> for CrateSearchQuery {
  fn from(search_term: String) -> Self {
    Self::from_term(search_term)
  }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Error)]
pub enum CrateError {
  #[error("Not logged in")]
  NotLoggedIn,
  #[error("Crate was not found")]
  NotFound,
  #[error("Internal server error")]
  Internal,
}

#[cfg(feature = "http_status_code")]
pub mod http_status_code {
  use crate::util::http_status_code::{AsStatusCode, StatusCode};

  use super::CrateError;

  impl AsStatusCode for CrateError {
    #[inline]
    fn as_status_code(&self) -> StatusCode {
      match self {
        Self::NotLoggedIn => StatusCode::FORBIDDEN,
        Self::NotFound => StatusCode::NOT_FOUND,
        Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
      }
    }
  }
}
