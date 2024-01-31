use iced::{Command, Element};
use iced::widget::text_input;
use tracing::instrument;

use att_client::http_client::AttHttpClient;
use att_client::search_crates::{SearchCrates, SearchCratesRequest, SearchCratesResponse};
use att_core::crates::Crate;

use crate::update::{OptPerformInto, Update};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct SearchCratesComponent {
  search_term_id: text_input::Id,
  choose_button_text: String,
  search_crates: SearchCrates,
}

#[derive(Debug)]
pub enum Message {
  SendRequest(SearchCratesRequest),
  ProcessResponse(SearchCratesResponse),
  Choose(String),
}

impl SearchCratesComponent {
  pub fn new(http_client: AttHttpClient, button_text: impl Into<String>) -> Self {
    Self {
      search_term_id: text_input::Id::unique(),
      choose_button_text: button_text.into(),
      search_crates: SearchCrates::new(http_client)
    }
  }

  pub fn focus_search_term_input<M: 'static>(&self) -> Command<M> {
    text_input::focus(self.search_term_id.clone())
  }

  pub fn clear(&mut self) {
    self.search_crates.clear();
  }
}

impl SearchCratesComponent {
  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<Option<String>, Command<Message>> {
    use Message::*;
    match message {
      SendRequest(request) => return self.search_crates.send(request).opt_perform_into(ProcessResponse).into(),
      ProcessResponse(response) => if let Some(request) = self.search_crates.process(response) {
        return self.search_crates.send(request).opt_perform_into(ProcessResponse).into();
      }
      Choose(crate_id) => return Update::from_action(Some(crate_id)),
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let crates = self.search_crates.found_crates();
    let cell_to_element = |row, col| -> Option<Element<Message>> {
      let Some(krate): Option<&Crate> = crates.get(row) else { return None; };
      let element = match col {
        0 => WidgetBuilder::once().add_text(&krate.id),
        1 => WidgetBuilder::once().add_text(&krate.max_version),
        2 => WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string()),
        3 => WidgetBuilder::once().add_text(format!("{}", krate.downloads)),
        4 => WidgetBuilder::once().button(self.choose_button_text.as_str()).padding([1.0, 5.0]).positive_style().on_press(|| Message::Choose(krate.id.clone())).add(),
        _ => return None,
      };
      Some(element)
    };
    let crates_table = Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(crates.len())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(1, "")
      .into_element();

    WidgetBuilder::stack()
      .text_input("Crate search term", self.search_crates.search_term())
      .id(self.search_term_id.clone())
      .on_input(|search_term| Message::SendRequest(self.search_crates.request_set_search_term(search_term)))
      .add()
      .add_element(crates_table)
      .column().spacing(20).width(800).height(600).add()
      .take()
  }
}
