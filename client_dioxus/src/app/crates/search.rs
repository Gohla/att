use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use att_client::http_client::AttHttpClient;
use att_client::search_crates::SearchCrates;
use att_core::crates::Crate;

use crate::app::crates::render_crates_table;
use crate::hook::context::UseContextExt;
use crate::hook::prelude::UseValueExt;
use crate::hook::request::UseRequestExt;

#[component]
pub fn SearchCratesComponent<FC: Fn(), FF: Fn(String)>(cx: Scope, handle_close: FC, handle_follow: FF) -> Element {
  let http_client: &AttHttpClient = cx.use_context_unwrap();

  let search_crates = cx.use_value(|| SearchCrates::new(http_client.clone()));
  let (request_tx, response_rx) = cx.use_request_opt(64, |r| search_crates.get_mut().send(r));
  for response in response_rx.drain() {
    if let Some(request) = search_crates.get_mut().process(response) {
      request_tx.send(request);
    }
  }

  let table = render_crates_table(cx, search_crates.get().found_crates().iter(), &|krate: &Crate| {
    render! {
      button {
        onclick: |event| {
          if let Some(MouseButton::Primary) = event.trigger_button() {
            handle_follow(krate.id.clone())
          }
        },
        "Follow"
      }
    }
  });

  render! {
    h2 { "Search" }
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
    table
  }
}
