use std::future::Future;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use futures::FutureExt;

use att_client::{AttClient, CratesData, CratesViewData, RemoveCrate, UpdateCrate, UpdateCrates};

use crate::hook::prelude::*;

enum Action {
  GetFollowedCrates,
  RefreshCrate(String),
  RefreshOutdatedCrates,
  RefreshAllCrates,
  UnfollowCrate(String),
}
impl Action {
  fn run(self, client: AttClient, view_data: &mut CratesViewData) -> impl Future<Output=Operation> {
    use Action::*;
    use Operation::*;
    match self {
      GetFollowedCrates => client.get_followed_crates(view_data).map(SetCrates).boxed_local(),
      RefreshCrate(crate_id) => client.refresh_crate(view_data, crate_id).map(UpdateCrate).boxed_local(),
      RefreshOutdatedCrates => client.refresh_outdated_crates(view_data).map(UpdateCrates).boxed_local(),
      RefreshAllCrates => client.refresh_all_crates(view_data).map(SetCrates).boxed_local(),
      UnfollowCrate(crate_id) => client.unfollow_crate(view_data, crate_id).map(RemoveCrate).boxed_local(),
    }
  }
}

enum Operation {
  UpdateCrate(UpdateCrate),
  UpdateCrates(UpdateCrates<false>),
  SetCrates(UpdateCrates<true>),
  RemoveCrate(RemoveCrate),
}
impl Operation {
  fn apply(self, view_data: &mut CratesViewData, data: &mut CratesData) {
    match self {
      Operation::UpdateCrate(operation) => { let _ = operation.apply(view_data, data); }
      Operation::UpdateCrates(operation) => { let _ = operation.apply(view_data, data); }
      Operation::SetCrates(operation) => { let _ = operation.apply(view_data, data); }
      Operation::RemoveCrate(operation) => { let _ = operation.apply(view_data, data); }
    }
  }
}

#[component]
pub fn ViewFollowedCrates(cx: Scope) -> Element {
  let client: &AttClient = cx.use_context_unwrap();

  let view_data = cx.use_value_default();
  let data = cx.use_value_default();

  let actions = cx.use_future(64, |action: Action| action.run(client.clone(), view_data.get_mut()));
  for operation in actions.iter_take() {
    operation.apply(view_data.get_mut(), data.get_mut());
  }
  let actions_handle = actions.run_handle();

  cx.use_once(|| actions_handle.run(Action::GetFollowedCrates));

  let disable_refresh = view_data.get().is_any_crate_being_modified();
  render! {
    h2 { "Followed Crates" }
    div {
      button { "Add" }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            actions_handle.run(Action::RefreshOutdatedCrates);
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            actions_handle.run(Action::RefreshAllCrates);
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
                      actions_handle.run(Action::RefreshCrate(krate.id.clone()));
                    }
                  },
                  disabled: disabled,
                  "Refresh"
                }
                button {
                  onclick: move |event| {
                    if let Some(MouseButton::Primary) = event.trigger_button() {
                      actions_handle.run(Action::UnfollowCrate(krate.id.clone()));
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
