use serde::{Deserialize, Serialize};

use crate::krate::CrateData;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
  pub crate_data: CrateData,
}
