use serde::{Deserialize, Serialize};

use crate::krate::CratesData;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub crates: CratesData,
}
