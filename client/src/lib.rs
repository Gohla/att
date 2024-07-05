use serde::{Deserialize, Serialize};

use crates::CratesState;

pub mod http_client;
pub mod auth;
pub mod crates;
pub mod search_crates;
pub mod query_sender;

#[derive(Default, Debug, Deserialize)]
pub struct Data {
  pub follow_crates: CratesState,
}

#[derive(Debug, Serialize)]
pub struct DataRef<'a> {
  pub follow_crates: &'a CratesState,
}
