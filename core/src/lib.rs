use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Crate {
  pub id: String,
  pub downloads: u64,
  pub updated_at: DateTime<Utc>,
  pub max_version: String,
}
