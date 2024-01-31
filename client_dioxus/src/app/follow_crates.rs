use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::follow_crates::{FollowCrateRequest, FollowCrates};
use att_client::http_client::AttHttpClient;

use crate::hook::prelude::*;

#[component]
pub fn FollowCrates(cx: Scope) -> Element {
  let http_client: &AttHttpClient = cx.use_context_unwrap();

  let follow_crates = cx.use_value(|| FollowCrates::new(http_client.clone()));
  let data = cx.use_value_default();

  let requests = cx.use_future(64, |r| follow_crates.get_mut().send(r));
  for response in requests.drain_values() {
    follow_crates.get_mut().process(response, data.get_mut());
  }
  let request_handle = requests.handle();

  cx.use_once(|| request_handle.run(FollowCrateRequest::GetFollowed));

  let disable_refresh = follow_crates.get().is_any_crate_being_modified();
  render! {
    h2 { "Followed Crates" }
    div {
      button { "Add" }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            request_handle.run(FollowCrateRequest::RefreshOutdated);
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            request_handle.run(FollowCrateRequest::RefreshAll);
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
        data.get().followed_crates().map(|krate|{
          let disabled = follow_crates.get().is_crate_being_modified(&krate.id);
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
                      request_handle.run(FollowCrateRequest::Refresh(krate.id.clone()));
                    }
                  },
                  disabled: disabled,
                  "Refresh"
                }
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      request_handle.run(FollowCrateRequest::Unfollow(krate.id.clone()));
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
