use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::follow_crates::{FollowCrateRequest, FollowCrates};
use att_client::http_client::AttHttpClient;
use att_core::crates::Crate;

use crate::app::crates::CratesTable;
use crate::app::crates::search::SearchCratesComponent;
use crate::hook::prelude::*;

#[component]
pub fn FollowCratesComponent(cx: Scope) -> Element {
  let http_client: &AttHttpClient = cx.use_context_unwrap();

  let follow_crates = cx.use_value(|| FollowCrates::new(http_client.clone()));
  let follow_crates_data = cx.use_value_default();
  let (follow_crates_request_tx, follow_crates_response_rx) = cx.use_request(8, |r| follow_crates.get_mut().send(r));
  cx.use_once(|| follow_crates_request_tx.send(FollowCrateRequest::GetFollowed));
  for response in follow_crates_response_rx.drain() {
    follow_crates.get_mut().process(response, follow_crates_data.get_mut());
  }

  let search_open = use_state(cx, || false);

  let search = if *search_open.get() {
    let nodes = rsx! {
      SearchCratesComponent {
        header: "Follow New Crate".to_string(),
        handle_close: || {
          search_open.set(false);
        },
        choose_button_text: "Follow".to_string(),
        handle_choose: |crate_id| {
          follow_crates_request_tx.send(FollowCrateRequest::Follow(crate_id));
          search_open.set(false);
        },
      }
    };
    Some(nodes)
  } else {
    None
  };

  let disable_refresh = follow_crates.get().is_any_crate_being_modified();
  render! {
    h2 { "Followed Crates" }
    div {
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            search_open.set(true);
          }
        },
        "Add"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            follow_crates_request_tx.send(FollowCrateRequest::RefreshOutdated);
          }
        },
        disabled: disable_refresh,
        "Refresh Outdated Crates"
      }
      button {
        onclick: move |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            follow_crates_request_tx.send(FollowCrateRequest::RefreshAll);
          }
        },
        disabled: disable_refresh,
        "Refresh All Crates"
      }
    }
    CratesTable {
      get_crates: || follow_crates_data.get().followed_crates(),
      render_actions: move |krate: &Crate| {
        let disabled = follow_crates.get().is_crate_being_modified(&krate.id);
        rsx! {
          button {
            onclick: move |event| {
              if let Some(MouseButton::Primary) = event.trigger_button() {
                follow_crates_request_tx.send(FollowCrateRequest::Refresh(krate.id.clone()));
              }
            },
            disabled: disabled,
            "Refresh"
          }
          button {
            onclick: move |event| {
              if let Some(MouseButton::Primary) = event.trigger_button() {
                follow_crates_request_tx.send(FollowCrateRequest::Unfollow(krate.id.clone()));
              }
            },
            disabled: disabled,
            "Unfollow"
          }
        }
      }
    }
    search
  }
}
