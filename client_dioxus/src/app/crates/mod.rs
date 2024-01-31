use dioxus::core::{Element, Scope};
use dioxus::core_macro::render;
use dioxus::prelude::*;

use att_core::crates::Crate;

pub mod follow;
pub mod search;

pub fn render_crates_table<'a, P>(
  cx: Scope<'a, P>,
  crates: impl Iterator<Item=&'a Crate>,
  render_actions: &impl Fn(&'a Crate) -> Element<'a>
) -> Element<'a> {
  render! {
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
        crates.map(|krate|{
          rsx! {
            tr {
              key: "{krate.id}",
              td { "{krate.id}" }
              td { "{krate.downloads}" }
              td { "{krate.updated_at}" }
              td { "{krate.max_version}" }
              td {
                render_actions(krate)
              }
            }
          }
        })
      }
    }
  }
}
