use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::AttClient;
use att_client::crates::{CrateAction, CrateRequest};

use crate::hook::prelude::*;

#[component]
pub fn ViewFollowedCrates(cx: Scope) -> Element {
  let client: &AttClient = cx.use_context_unwrap();
  let request: CrateRequest = client.crates();

  let view_data = cx.use_value_default();
  let data = cx.use_value_default();

  let actions = cx.use_future(64, |action: CrateAction| action.perform(request.clone(), view_data.get_mut()));
  for operation in actions.iter_take() {
    operation.apply(view_data.get_mut(), data.get_mut());
  }
  let actions_handle = actions.run_handle();

  cx.use_once(|| actions_handle.run(CrateAction::GetFollowed));

  let disable_refresh = view_data.get().is_any_crate_being_modified();
  render! {
    h2 { "Followed Crates" }
    div {
      button { "Add" }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            actions_handle.run(CrateAction::RefreshOutdated);
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            actions_handle.run(CrateAction::RefreshAll);
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
                      actions_handle.run(CrateAction::Refresh(krate.id.clone()));
                    }
                  },
                  disabled: disabled,
                  "Refresh"
                }
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      actions_handle.run(CrateAction::Unfollow(krate.id.clone()));
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
