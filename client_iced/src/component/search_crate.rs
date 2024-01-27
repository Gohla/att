use std::time::Duration;

use iced::{Command, Element};
use iced::widget::text_input;
use tracing::{error, instrument};

use att_client::http_client::{AttHttpClient, AttHttpClientError};
use att_core::crates::{Crate, CrateSearch};
use att_core::util::time::{Instant, sleep};

use crate::component::{Perform, Update};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

#[derive(Debug)]
pub struct SearchCrates {
  text_input_id: text_input::Id,
  button_text: String,

  search_term: String,
  wait_until: Option<Instant>,
  result: Result<Vec<Crate>, AttHttpClientError>,
}

#[derive(Debug)]
pub enum Message {
  SetSearchTerm(String),
  Search,
  SetResult(Result<Vec<Crate>, AttHttpClientError>),
  Choose(String),
}

impl Default for SearchCrates {
  fn default() -> Self {
    Self {
      text_input_id: text_input::Id::unique(),
      button_text: "Choose".to_string(),

      search_term: String::new(),
      wait_until: None,
      result: Ok(vec![]),
    }
  }
}

impl SearchCrates {
  pub fn new(button_text: impl Into<String>) -> Self {
    Self { button_text: button_text.into(), ..Self::default() }
  }

  pub fn focus_search_term_input<M: 'static>(&self) -> Command<M> {
    text_input::focus(self.text_input_id.clone())
  }

  pub fn clear(&mut self) {
    self.search_term.clear();
    self.wait_until = None;
    self.result = Ok(vec![]);
  }
}

impl SearchCrates {
  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message, client: &AttHttpClient) -> Update<Option<String>, Command<Message>> {
    use Message::*;
    match message {
      SetSearchTerm(search_term) => {
        self.search_term = search_term.clone();
        return if !search_term.is_empty() {
          let wait_duration = Duration::from_millis(300);
          let wait_until = Instant::now() + wait_duration;
          self.wait_until = Some(wait_until);
          sleep(wait_duration).perform(|_| Search).into()
        } else {
          self.wait_until = None;
          self.result = Ok(vec![]);
          Update::empty()
        };
      }
      Search => if let Some(search_wait_until) = self.wait_until {
        if Instant::now() > search_wait_until {
          return client.clone().search_crates(CrateSearch::from_term(self.search_term.clone()))
            .perform(SetResult).into();
        }
      }
      SetResult(crates) => {
        if let Err(cause) = &crates {
          error!(%cause, "failed to search for crates: {cause:?}");
        }
        self.result = crates
      },
      Choose(crate_id) => return Update::from_action(Some(crate_id)),
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let builder = WidgetBuilder::stack()
      .text_input("Crate search term", &self.search_term).id(self.text_input_id.clone()).on_input(Message::SetSearchTerm).add();

    let crates = match &self.result {
      Ok(crates) => {
        let cell_to_element = |row, col| -> Option<Element<Message>> {
          let Some(krate): Option<&Crate> = crates.get(row) else { return None; };
          let element = match col {
            0 => WidgetBuilder::once().add_text(&krate.id),
            1 => WidgetBuilder::once().add_text(&krate.max_version),
            2 => WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string()),
            3 => WidgetBuilder::once().add_text(format!("{}", krate.downloads)),
            4 => WidgetBuilder::once().button(self.button_text.as_str()).padding([1.0, 5.0]).positive_style().on_press(|| Message::Choose(krate.id.clone())).add(),
            _ => return None,
          };
          Some(element)
        };
        Table::with_capacity(5, cell_to_element)
          .spacing(1.0)
          .body_row_height(24.0)
          .body_row_count(crates.len())
          .push(2, "Name")
          .push(1, "Latest Version")
          .push(1, "Updated at")
          .push(1, "Downloads")
          .push(1, "")
          .into_element()
      }
      Err(e) => WidgetBuilder::once().add_text(format!("{:?}", e)),
    };

    builder
      .add_element(crates)
      .column().spacing(20).width(800).height(600).add()
      .take()
  }
}
