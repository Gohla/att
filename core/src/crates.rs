use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub mod crates_io {
  use super::Crate;

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
  use crate::util::status_code::{AsStatusCode, StatusCode};

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
