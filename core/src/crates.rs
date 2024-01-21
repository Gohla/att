use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util;

#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CrateSearch {
  pub search_term: Option<String>,
  pub followed: bool,
}
impl CrateSearch {
  pub fn from_term(search_term: String) -> Self {
    Self { search_term: Some(search_term), ..Self::default() }
  }
  pub fn followed() -> Self {
    Self { followed: true, ..Self::default() }
  }
}
impl From<String> for CrateSearch {
  fn from(search_term: String) -> Self {
    Self::from_term(search_term)
  }
}


#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Crate {
  pub id: String,
  pub downloads: u64,
  pub updated_at: DateTime<Utc>,
  pub max_version: String,
}
impl Crate {
  pub fn from_id(id: String) -> Self {
    Self { id, ..Self::default() }
  }
}

#[cfg(feature = "crates_io")]
impl From<crates_io_api::Crate> for Crate {
  fn from(c: crates_io_api::Crate) -> Self {
    Self {
      id: c.id,
      downloads: c.downloads,
      updated_at: c.updated_at,
      max_version: c.max_version,
    }
  }
}
#[cfg(feature = "crates_io")]
impl From<&crates_io_api::Crate> for Crate {
  fn from(c: &crates_io_api::Crate) -> Self {
    Self {
      id: c.id.clone(),
      downloads: c.downloads,
      updated_at: c.updated_at,
      max_version: c.max_version.clone(),
    }
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
impl util::status_code::AsStatusCode for CrateError {
  #[inline]
  fn as_status_code(&self) -> util::status_code::StatusCode {
    match self {
      Self::NotLoggedIn => util::status_code::StatusCode::FORBIDDEN,
      Self::NotFound => util::status_code::StatusCode::NOT_FOUND,
      Self::Internal => util::status_code::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}
