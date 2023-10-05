use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(feature = "crates_io_api")]
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
#[cfg(feature = "crates_io_api")]
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
