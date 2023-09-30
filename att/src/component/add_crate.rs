use std::time::Duration;

use crates_io_api::{Crate, CratesPage};
use iced::{Command, Element};
use iced::widget::text_input;

use crate::component::Update;
use crate::crates_client::CratesClient;
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

/// Search for a crate on crates.io and add it.
#[derive(Debug)]
pub struct AddCrate {
  wait_before_searching: Duration,
  search_id: text_input::Id,
  search_term: String,
  // next_search_time: Option<Instant>,
  crates: Option<Result<CratesPage, crates_io_api::Error>>,
}

#[derive(Debug)]
pub enum Message {
  SetSearchTerm(String),
  SetCrates(Option<Result<CratesPage, crates_io_api::Error>>),
  AddCrate(Crate),
}

impl Default for AddCrate {
  fn default() -> Self {
    Self {
      wait_before_searching: Duration::from_millis(200),
      search_id: text_input::Id::unique(),
      search_term: String::new(),
      crates: None,
    }
  }
}

impl AddCrate {
  pub fn wait_before_searching(&mut self, wait_before_searching: Duration) {
    self.wait_before_searching = wait_before_searching;
  }

  pub fn focus_search_term_input<M: 'static>(&self) -> Command<M> {
    text_input::focus(self.search_id.clone())
  }

  pub fn clear_search_term(&mut self) {
    self.search_term.clear();
    self.crates = None;
  }
}

impl AddCrate {
  pub fn update(&mut self, message: Message, crates_client: &CratesClient) -> Update<Option<Crate>, Command<Message>> {
    match message {
      Message::SetSearchTerm(s) => {
        self.search_term = s.clone();
        return Update::perform(crates_client.clone().search(s), |r| Message::SetCrates(r));
      }
      Message::SetCrates(crates) => if let Some(crates) = crates {
        self.crates = Some(crates)
      },
      Message::AddCrate(krate) => {
        return Update::from_action(krate)
      },
    }
    Update::empty()
  }

  pub fn view<'a>(&'a self) -> Element<'a, Message> {
    let builder = WidgetBuilder::stack()
      .text_input("Crate search term", &self.search_term).id(self.search_id.clone()).on_input(Message::SetSearchTerm).add();

    let crates = match &self.crates {
      Some(Ok(crates)) => {
        let cell_to_element = |row, col| -> Option<Element<'a, Message>> {
          let Some(krate): Option<&Crate> = crates.crates.get(row) else { return None; };
          let element = match col {
            0 => WidgetBuilder::once().add_text(&krate.id),
            1 => WidgetBuilder::once().add_text(&krate.max_version),
            2 => WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string()),
            3 => WidgetBuilder::once().add_text(format!("{}", krate.downloads)),
            4 => WidgetBuilder::once().button("Add").padding([1.0, 5.0]).positive_style().on_press(|| Message::AddCrate(krate.clone())).add(),
            _ => return None,
          };
          Some(element)
        };
        Table::with_capacity(5, cell_to_element)
          .spacing(1.0)
          .body_row_height(24.0)
          .body_row_count(crates.crates.len())
          .push(2, "Name")
          .push(1, "Latest Version")
          .push(1, "Updated at")
          .push(1, "Downloads")
          .push(1, "")
          .into_element()
      }
      Some(Err(e)) => WidgetBuilder::once().add_text(format!("{:?}", e)),
      _ => WidgetBuilder::once().add_space_fill_width(),
    };

    builder
      .add_element(crates)
      .column().spacing(20).width(800).height(600).add()
      .take()
  }
}
