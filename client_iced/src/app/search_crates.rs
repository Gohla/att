use iced::{Element, Task};
use iced::widget::text_input;
use tracing::instrument;

use att_client::http_client::AttHttpClient;
use att_client::search_crates::{SearchCrates, SearchCratesRequest, SearchCratesResponse};
use att_core::crates::FullCrate;
use iced_builder::WidgetBuilder;
use iced_virtual::table::Table;

use crate::perform::OptionPerformExt;
use crate::update::Update;
use crate::widget::IntoElement;

pub struct SearchCratesComponent {
  search_term_id: text_input::Id,
  choose_button_text: String,
  search_crates: SearchCrates,
}

#[derive(Debug)]
pub enum Message {
  SendRequest(SearchCratesRequest),
  ProcessResponse(SearchCratesResponse),
  Choose(FullCrate),
}

impl SearchCratesComponent {
  pub fn new(http_client: AttHttpClient, button_text: impl Into<String>) -> Self {
    Self {
      search_term_id: text_input::Id::unique(),
      choose_button_text: button_text.into(),
      search_crates: SearchCrates::new(http_client)
    }
  }

  pub fn focus_search_term_input<M: 'static>(&self) -> Task<M> {
    text_input::focus(self.search_term_id.clone())
  }

  pub fn clear(&mut self) {
    self.search_crates.clear();
  }
}

impl SearchCratesComponent {
  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<Option<FullCrate>, Task<Message>> {
    use Message::*;
    match message {
      SendRequest(request) => return self.search_crates.send(request).opt_perform_into(ProcessResponse).into(),
      ProcessResponse(response) => if let Some(request) = self.search_crates.process(response) {
        return self.search_crates.send(request).opt_perform_into(ProcessResponse).into();
      }
      Choose(full_crate) => return Update::from_action(Some(full_crate)),
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let full_crates = self.search_crates.found_crates();
    let cell_to_element = |row, col| -> Option<Element<Message>> {
      let Some(full_crate): Option<&FullCrate> = full_crates.get(row) else { return None; };
      let element = match col {
        0 => WidgetBuilder::once().add_text(format!("{}", full_crate.krate.id)),
        1 => WidgetBuilder::once().add_text(&full_crate.krate.name),
        2 => WidgetBuilder::once().add_text(full_crate.krate.updated_at.format("%Y-%m-%d").to_string()),
        3 => WidgetBuilder::once().add_text(&full_crate.default_version.number),
        4 => WidgetBuilder::once().add_text(format!("{}", full_crate.krate.downloads)),
        5 => WidgetBuilder::once().add_text(&full_crate.krate.description),
        6 => WidgetBuilder::once().button(self.choose_button_text.as_str()).padding([1.0, 5.0]).success_style().on_press(|| Message::Choose(full_crate.clone())).add(),
        _ => return None,
      };
      Some(element)
    };
    let crates_table = Table::with_capacity(7, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(full_crates.len())
      .push(0.5, "Id")
      .push(1.0, "Name")
      .push(1.0, "Updated at")
      .push(1.0, "Latest Version")
      .push(1.0, "Downloads")
      .push(2.0, "Description")
      .push(1.0, "")
      .into_element();

    WidgetBuilder::stack()
      .text_input("Crate search term", self.search_crates.search_term())
      .id(self.search_term_id.clone())
      .on_input(|search_term| Message::SendRequest(self.search_crates.request_set_search_term(search_term))).add()
      .add_element(crates_table)
      .column().spacing(20).width(800).height(600).add()
      .take()
  }
}
