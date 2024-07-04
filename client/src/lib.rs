use serde::{Deserialize, Serialize};

use follow_crates::FollowCratesState;

pub mod http_client;
pub mod auth;
pub mod follow_crates;
pub mod search_crates;
pub mod search_query;

#[derive(Default, Debug, Deserialize)]
pub struct Data {
  pub follow_crates: FollowCratesState,
}

#[derive(Debug, Serialize)]
pub struct DataRef<'a> {
  pub follow_crates: &'a FollowCratesState,
}
