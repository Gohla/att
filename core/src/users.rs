use std::fmt::{Debug, Formatter};
use std::fmt;

use dotenvy_macro::dotenv;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCredentials {
  pub name: String,
  pub password: String,
}

impl UserCredentials {
  pub fn new(name: impl Into<String>, password: impl Into<String>) -> Self {
    Self { name: name.into(), password: password.into() }
  }
}

impl Default for UserCredentials {
  fn default() -> Self {
    UserCredentials::new(dotenv!("ATT_DEFAULT_USER_NAME"), dotenv!("ATT_DEFAULT_USER_PASSWORD"))
  }
}

impl Debug for UserCredentials {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("UserCredentials")
      .field("name", &self.name)
      .field("password", &"[redacted]")
      .finish()
  }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Error)]
pub enum AuthError {
  #[error("Incorrect user name or password")]
  IncorrectUserNameOrPassword,
  #[error("Internal server error")]
  Internal,
}

#[cfg(feature = "http_status_code")]
pub mod http_status_code {
  use crate::util::http_status_code::{AsStatusCode, StatusCode};

  use super::AuthError;

  impl AsStatusCode for AuthError {
    #[inline]
    fn as_status_code(&self) -> StatusCode {
      match self {
        Self::IncorrectUserNameOrPassword => StatusCode::FORBIDDEN,
        Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
      }
    }
  }
}
