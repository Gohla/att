use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::http_client::AttHttpClient;
use att_client::search_crates::SearchCrates;
use att_core::crates::Crate;

use crate::app::crates::CratesTable;
use crate::hook::context::UseContextExt;
use crate::hook::prelude::UseValueExt;
use crate::hook::request::UseRequestExt;

#[component]
pub fn SearchCratesComponent<HC: Fn(), HF: Fn(String)>(
  cx: Scope,
  header: String,
  handle_close: HC,
  choose_button_text: String,
  handle_choose: HF
) -> Element {
  let http_client: &AttHttpClient = cx.use_context_unwrap();

  let search_crates = cx.use_value(|| SearchCrates::new(http_client.clone()));
  let (request_tx, response_rx) = cx.use_request_opt(64, |r| search_crates.get_mut().send(r));
  for response in response_rx.drain() {
    if let Some(request) = search_crates.get_mut().process(response) {
      request_tx.send(request);
    }
  }

  render! {
    h2 { "{header}" }
    div {
      input {
        oninput: |event| {
          request_tx.send(search_crates.get().request_set_search_term(event.value.clone()))
        }
      }
      button {
        onclick: |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            handle_close()
          }
        },
        "Close"
      }
    }
    CratesTable {
      get_crates: || search_crates.get().found_crates().iter(),
      render_actions: move |krate: &Crate| {
        rsx! {
          button {
            onclick: |event| {
              if let Some(MouseButton::Primary) = event.trigger_button() {
                handle_choose(krate.id.clone())
              }
            },
            "{choose_button_text}"
          }
        }
      }
    }
  }
}
