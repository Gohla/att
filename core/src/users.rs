use std::fmt::{Debug, Formatter};
use std::fmt;
use dotenvy_macro::dotenv;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCredentials {
  pub name: String,
  pub password: String,
}
impl Default for UserCredentials {
  fn default() -> Self {
    UserCredentials::new(dotenv!("ATT_DEFAULT_USER_NAME"), dotenv!("ATT_DEFAULT_USER_PASSWORD"))
  }
}
impl UserCredentials {
  pub fn new(name: impl Into<String>, password: impl Into<String>) -> Self {
    Self { name: name.into(), password: password.into() }
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
