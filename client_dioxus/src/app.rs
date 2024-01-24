use dioxus::prelude::*;

use att_client::{AttClient, Data};
use att_core::crates::Crate;

pub struct AppProps {
  data: Data,
  client: AttClient,
}
impl AppProps {
  pub fn new(data: Data, client: AttClient) -> Self {
    Self { data, client }
  }
}

#[component]
pub fn App(cx: Scope<AppProps>) -> Element {
  render! { "test" }
  // //let AppProps { data, client: _client } = cx.props;
  // cx.render(rsx! {
  //   "test"
  //   // h1 { "Crates" }
  //   // div {
  //   //   for krate in data.id_to_crate.values() {
  //   //     Krate { krate: krate }
  //   //   }
  //   // }
  // })
}

// #[component]
// fn Krate<'a>(cx: Scope, krate: &'a Crate) -> Element {
//   cx.render(rsx! {
//     div {
//       padding: "1.0rem",
//       position: "relative",
//       color: "red",
//       "{krate.id} | downloads: {krate.downloads}, updated at: {krate.updated_at}, max version: {krate.max_version}"
//     }
//   })
// }
