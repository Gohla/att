use dioxus::prelude::*;

use att_client::{AttClient, LoginState};
use att_core::crates::Crate;
use att_core::users::UserCredentials;

use crate::hook::future::UseFutureOnceExt;
use crate::hook::value::UseValueExt;

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
  let client = &cx.props.client;

  let view_data = cx.use_value_default();

  let login = cx.use_future_once(|| client.clone().login(view_data.get_mut(), UserCredentials::default()));
  if let Some(login) = login.get() {
    let _ = login.apply(view_data.get_mut());
  }

  let body = match view_data.get().login_state() {
    LoginState::LoggedOut => rsx! { "Logged out" },
    LoginState::LoggedIn => rsx! { Crates { client: client } },
    LoginState::LoggingIn => rsx! { "Logging in" },
    LoginState::LoggingOut => rsx! { "Logging out" },
  };
  render! {
    h1 { "All The Things!" }
    body
  }
}

#[component]
fn Crates<'a>(cx: Scope<'a>, client: &'a AttClient) -> Element<'a> {
  let client = (*client).clone();

  let view_data = cx.use_value_default();
  let data = cx.use_value_default();

  let update_crates = cx.use_future_once(|| client.clone().get_followed_crates(view_data.get_mut()));
  if let Some(update_crates) = update_crates.get() {
    let _ = update_crates.apply(view_data.get_mut(), data.get_mut());
  }

  render! {
    h2 { "Followed Crates" }
    table {
      thead {
        tr {
          th { "Name" }
          th { "Downloads" }
          th { "Updated at" }
          th { "Max version" }
        }
      }
      tbody {
        for krate in data.get().id_to_crate.values() {
          Crate { key: "{krate.id}", krate: krate.clone() }
        }
      }
    }
  }
}

#[component]
fn Crate(cx: Scope, krate: Crate) -> Element {
  render! {
    tr {
      td { "{krate.id}" }
      td { "{krate.downloads}" }
      td { "{krate.updated_at}" }
      td { "{krate.max_version}" }
    }
  }
}
