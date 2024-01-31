use dioxus::prelude::*;

use att_client::auth::{Auth, AuthStatus};
use att_client::http_client::AttHttpClient;
use att_core::users::UserCredentials;

use crate::app::crates::follow::FollowCrates;
use crate::hook::prelude::*;

mod crates;

pub struct AppProps {
  http_client: AttHttpClient,
}
impl AppProps {
  pub fn new(http_client: AttHttpClient) -> Self {
    Self { http_client }
  }
}

#[component]
pub fn App(cx: Scope<AppProps>) -> Element {
  let http_client = cx.use_context_provider(&cx.props.http_client);

  let auth = cx.use_value(|| Auth::new(http_client.clone()));

  let login = cx.use_future_once(|| auth.get_mut().login(UserCredentials::default()));
  if let Some(logged_in) = login.try_take() {
    let _ = auth.get_mut().process_logged_in(logged_in);
  }

  let body = match auth.get().status() {
    AuthStatus::LoggedOut => rsx! { "Logged out" },
    AuthStatus::LoggedIn => rsx! { FollowCrates {} },
    AuthStatus::LoggingIn => rsx! { "Logging in" },
    AuthStatus::LoggingOut => rsx! { "Logging out" },
  };
  render! {
    h1 { "All The Things!" }
    body
  }
}
