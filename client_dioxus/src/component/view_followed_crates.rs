use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::AttClient;
use att_client::crates::{CrateClient, CrateRequest};

use crate::hook::prelude::*;

#[component]
pub fn ViewFollowedCrates(cx: Scope) -> Element {
  let client: &AttClient = cx.use_context_unwrap();
  let client: CrateClient = client.clone().into_crate_client();

  let view_data = cx.use_value_default();
  let data = cx.use_value_default();

  let responses = cx.use_future(64, |request: CrateRequest| request.send(&client, view_data.get_mut()));
  for response in responses.iter_take() {
    response.process(view_data.get_mut(), data.get_mut());
  }
  let request_handle = responses.run_handle();

  cx.use_once(|| request_handle.run(CrateRequest::GetFollowed));

  let disable_refresh = view_data.get().is_any_crate_being_modified();
  render! {
    h2 { "Followed Crates" }
    div {
      button { "Add" }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            request_handle.run(CrateRequest::RefreshOutdated);
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            request_handle.run(CrateRequest::RefreshAll);
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
                      request_handle.run(CrateRequest::Refresh(krate.id.clone()));
                    }
                  },
                  disabled: disabled,
                  "Refresh"
                }
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      request_handle.run(CrateRequest::Unfollow(krate.id.clone()));
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
