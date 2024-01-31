use serde::{Deserialize, Serialize};

use follow_crates::FollowCratesData;

pub mod http_client;
pub mod auth;
pub mod follow_crates;
pub mod search_crates;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  crates: FollowCratesData,
}
impl Data {
  #[inline]
  pub fn crates(&self) -> &FollowCratesData { &self.crates }
  #[inline]
  pub fn crates_mut(&mut self) -> &mut FollowCratesData { &mut self.crates }
}
