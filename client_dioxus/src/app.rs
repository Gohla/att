use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::{AttClient, LoginState};
use att_core::users::UserCredentials;

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
  let client = &cx.props.client;

  let view_data = cx.use_value_default();

  let login = cx.use_future_once(|| client.clone().login(view_data.get_mut(), UserCredentials::default()));
  if let Some(login) = login.try_take() {
    let _ = login.apply(view_data.get_mut());
  }

  let body = match view_data.get().login_state() {
    LoginState::LoggedOut => rsx! { "Logged out" },
    LoginState::LoggedIn => rsx! { ViewFollowedCrates { client: client } },
    LoginState::LoggingIn => rsx! { "Logging in" },
    LoginState::LoggingOut => rsx! { "Logging out" },
  };
  render! {
    h1 { "All The Things!" }
    body
  }
}

#[component]
fn ViewFollowedCrates<'a>(cx: Scope<'a>, client: &'a AttClient) -> Element<'a> {
  let client = (*client).clone();

  let view_data = cx.use_value_default();
  let data = cx.use_value_default();

  let update_crates_once = cx.use_future_once(|| client.clone().get_followed_crates(view_data.get_mut()));
  if let Some(operation) = update_crates_once.try_take() {
    let _ = operation.apply(view_data.get_mut(), data.get_mut());
  }
  let refresh_outdated_crates = cx.use_future_single(|| client.clone().refresh_outdated_crates(view_data.get_mut()));
  if let Some(operation) = refresh_outdated_crates.try_take() {
    let _ = operation.apply(view_data.get_mut(), data.get_mut());
  }
  let refresh_all_crates = cx.use_future_single(|| client.clone().refresh_all_crates(view_data.get_mut()));
  if let Some(operation) = refresh_all_crates.try_take() {
    let _ = operation.apply(view_data.get_mut(), data.get_mut());
  }
  let refresh_crate = cx.use_future(64, |crate_id| client.clone().refresh_crate(view_data.get_mut(), crate_id));
  for operation in refresh_crate.iter_take() {
    let _ = operation.apply(view_data.get_mut(), data.get_mut());
  }
  let unfollow_crate = cx.use_future(64, |crate_id| client.clone().unfollow_crate(view_data.get_mut(), crate_id));
  for operation in unfollow_crate.iter_take() {
    let _ = operation.apply(view_data.get_mut(), data.get_mut());
  }

  let disable_refresh = view_data.get().is_any_crate_being_modified();

  render! {
    h2 { "Followed Crates" }
    div {
      button { "Add" }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            refresh_outdated_crates.run();
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            refresh_all_crates.run();
          }
        },
        disabled: disable_refresh,
        "Refresh All Crates"
      }
    }
    table {
      thead {
        tr {
          th { "Name" }
          th { "Downloads" }
          th { "Updated at" }
          th { "Max version" }
          th { "Actions" }
        }
      }
      tbody {
        data.get().id_to_crate.values().map(|krate|{
          let refresh_handle = refresh_crate.run_handle();
          let unfollow_handle = unfollow_crate.run_handle();
          let disabled = view_data.get().is_crate_being_modified(&krate.id);
          rsx! {
            tr {
              key: "{krate.id}",
              td { "{krate.id}" }
              td { "{krate.downloads}" }
              td { "{krate.updated_at}" }
              td { "{krate.max_version}" }
              td {
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      refresh_handle.run(krate.id.clone());
                    }
                  },
                  disabled: disabled,
                  "Refresh"
                }
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      unfollow_handle.run(krate.id.clone());
                    }
                  },
                  disabled: disabled,
                  "Unfollow"
                }
              }
            }
          }
        })
      }
    }
  }
}

