use dioxus::prelude::*;

pub fn app(cx: Scope) -> Element {
  let title = "title";
  let by = "author";
  let score = 0;
  let time = chrono::Utc::now();
  let comments = "comments";

  cx.render(rsx! {
    div {
      padding: "1.0rem",
      position: "relative",
      color: "red",
      "{title} by {by} ({score}) {time} {comments}"
    }
  })
}
