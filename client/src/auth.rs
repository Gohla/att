use std::future::Future;

use tracing::{debug, error};

use att_core::users::UserCredentials;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Authentication status.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum AuthStatus {
  #[default] LoggedOut,
  LoggedIn,
  LoggingIn,
  LoggingOut,
}

/// Keep track of authentication status.
#[derive(Debug)]
pub struct Auth {
  http_client: AttHttpClient,
  status: AuthStatus,
}
impl Auth {
  #[inline]
  pub fn new(http_client: AttHttpClient) -> Self {
    Self { http_client, status: AuthStatus::default() }
  }

  #[inline]
  pub fn status(&self) -> &AuthStatus { &self.status }

  pub fn login(&mut self, user_credentials: UserCredentials) -> impl Future<Output=LoggedIn> {
    self.status = AuthStatus::LoggingIn;
    let future = self.http_client.login(user_credentials);
    async move {
      LoggedIn { result: future.await }
    }
  }
  pub fn process_logged_in(&mut self, response: LoggedIn) -> Result<(), AttHttpClientError> {
    self.status = AuthStatus::LoggedOut; // First reset.

    response.result
      .inspect_err(|cause| error!(%cause, "failed to login: {cause:?}"))?;
    debug!("logged in");
    self.status = AuthStatus::LoggedIn; // Only set if there is no error.

    Ok(())
  }

  pub fn logout(&mut self) -> impl Future<Output=LoggedOut> {
    self.status = AuthStatus::LoggingOut;
    let future = self.http_client.logout();
    async move {
      LoggedOut { result: future.await }
    }
  }
  pub fn process_logged_out(&mut self, response: LoggedOut) -> Result<(), AttHttpClientError> {
    self.status = AuthStatus::LoggedIn; // First reset.

    response.result
      .inspect_err(|cause| error!(%cause, "failed to logout: {cause:?}"))?;
    debug!("logged out");
    self.status = AuthStatus::LoggedOut; // Only set if there is no error.

    Ok(())
  }
}

/// Logged in response.
#[derive(Debug)]
pub struct LoggedIn {
  result: Result<(), AttHttpClientError>,
}

/// Logged out response.
#[derive(Debug)]
pub struct LoggedOut {
  result: Result<(), AttHttpClientError>,
}
