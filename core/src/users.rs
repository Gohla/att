use std::fmt::{Debug, Formatter};
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCredentials {
  pub name: String,
  pub password: String,
}
impl UserCredentials {
  pub fn new(name: String, password: String) -> Self {
    Self { name, password }
  }
}
impl Debug for UserCredentials {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("User")
      .field("name", &self.name)
      .field("password", &"[redacted]")
      .finish()
  }
}
