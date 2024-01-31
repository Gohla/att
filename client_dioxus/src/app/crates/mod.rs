use dioxus::core::{Element, Scope};
use dioxus::core_macro::render;
use dioxus::prelude::*;

use att_core::crates::Crate;

pub mod follow;
pub mod search;

#[component]
pub fn CratesTable<'a, 'b, GC: Fn() -> C, C: Iterator<Item=&'a Crate>, R: Fn(&'a Crate) -> LazyNodes<'a, 'b>>(
  cx: Scope<'a>,
  get_crates: GC,
  render_actions: R
) -> Element<'a> {
  let crates = get_crates();
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
        for krate in crates {
          tr {
            key: "{krate.id}",
            td { "{krate.id}" }
            td { "{krate.downloads}" }
            td { "{krate.updated_at}" }
            td { "{krate.max_version}" }
            td {
              (&render_actions)(krate)
            }
          }
        }
      }
    }
  }
}
