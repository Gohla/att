use dioxus::prelude::*;

use att_client::{AttClient, LoginState};
use att_core::users::UserCredentials;

use crate::component::view_followed_crates::ViewFollowedCrates;
use crate::hook::prelude::*;

pub struct AppProps {
  client: AttClient,
}
impl AppProps {
  pub fn new(client: AttClient) -> Self {
    Self { client }
  }
}

#[component]
pub fn App(cx: Scope<AppProps>) -> Element {
  let client = cx.use_context_provider(&cx.props.client);

  let view_data = cx.use_value_default();

  let login = cx.use_future_once(|| client.clone().login(view_data.get_mut(), UserCredentials::default()));
  if let Some(login) = login.try_take() {
    let _ = login.apply(view_data.get_mut());
  }

  let body = match view_data.get().login_state() {
    LoginState::LoggedOut => rsx! { "Logged out" },
    LoginState::LoggedIn => rsx! { ViewFollowedCrates {} },
    LoginState::LoggingIn => rsx! { "Logging in" },
    LoginState::LoggingOut => rsx! { "Logging out" },
  };
  render! {
    h1 { "All The Things!" }
    body
  }
}
