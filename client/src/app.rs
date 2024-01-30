use std::future::Future;

use tracing::{debug, error};

use att_core::users::UserCredentials;

use crate::http_client::{AttHttpClient, AttHttpClientError};

/// Application login state.
#[derive(Default)]
pub enum LoginState {
  #[default] LoggedOut,
  LoggedIn,
  LoggingIn,
  LoggingOut,
}

/// Application view data.
#[derive(Default)]
pub struct AppViewData {
  login_state: LoginState,
}
impl AppViewData {
  #[inline]
  pub fn login_state(&self) -> &LoginState { &self.login_state }
}


/// Application requests.
#[derive(Clone)]
pub struct AppRequest {
  http_client: AttHttpClient,
}
impl AppRequest {
  #[inline]
  pub(crate) fn new(http_client: AttHttpClient) -> Self { Self { http_client } }

  pub fn login(self, view_data: &mut AppViewData, user_credentials: UserCredentials) -> impl Future<Output=Login> {
    view_data.login_state = LoginState::LoggingIn;
    async move {
      let result = self.http_client.login(user_credentials).await;
      Login { result }
    }
  }
  pub fn logout(self, view_data: &mut AppViewData) -> impl Future<Output=Logout> {
    view_data.login_state = LoginState::LoggingOut;
    async move {
      let result = self.http_client.logout().await;
      Logout { result }
    }
  }
}


/// Application login operation.
#[derive(Debug)]
pub struct Login {
  result: Result<(), AttHttpClientError>,
}
impl Login {
  pub fn apply(self, view_data: &mut AppViewData) -> Result<(), AttHttpClientError> {
    view_data.login_state = LoginState::LoggedOut; // First reset.

    self.result
      .inspect_err(|cause| error!(%cause, "failed to login: {cause:?}"))?;
    debug!("logged in");
    view_data.login_state = LoginState::LoggedIn; // Only set if there is no error.

    Ok(())
  }
}

/// Application logout operation.
#[derive(Debug)]
pub struct Logout {
  result: Result<(), AttHttpClientError>,
}
impl Logout {
  pub fn apply(self, view_data: &mut AppViewData) -> Result<(), AttHttpClientError> {
    view_data.login_state = LoginState::LoggedIn; // First reset.

    self.result
      .inspect_err(|cause| error!(%cause, "failed to logout: {cause:?}"))?;
    debug!("logged out");
    view_data.login_state = LoginState::LoggedOut; // Only set if there is no error.

    Ok(())
  }
}
