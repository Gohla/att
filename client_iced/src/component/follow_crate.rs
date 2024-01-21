use std::time::Duration;

use iced::{Command, Element};
use iced::widget::text_input;
use tracing::{error, instrument};
use att_client::{AttHttpClient, AttHttpClientError};

use att_core::crates::{Crate, CrateSearch};

use crate::component::{Perform, Update};
use att_core::util::time::{Instant, sleep};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

#[derive(Debug)]
pub struct FollowCrate {
  search_id: text_input::Id,
  search_term: String,
  search_wait_until: Option<Instant>,
  crates: Result<Vec<Crate>, AttHttpClientError>,
}

#[derive(Default, Debug)]
pub enum Message {
  SetSearchTerm(String),
  SearchCrates,
  SetCrates(Result<Vec<Crate>, AttHttpClientError>),
  FollowCrate(String),
  ReceiveFollowedCrate(Result<Crate, AttHttpClientError>),
  #[default]
  Ignore,
}

impl Default for FollowCrate {
  fn default() -> Self {
    Self {
      search_id: text_input::Id::unique(),
      search_term: String::new(),
      search_wait_until: None,
      crates: Ok(vec![]),
    }
  }
}

impl FollowCrate {
  pub fn focus_search_term_input<M: 'static>(&self) -> Command<M> {
    text_input::focus(self.search_id.clone())
  }

  pub fn clear_search_term(&mut self) {
    self.search_term.clear();
    self.crates = Ok(vec![]);
  }
}

impl FollowCrate {
  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message, client: &AttHttpClient) -> Update<Option<Crate>, Command<Message>> {
    use Message::*;
    match message {
      SetSearchTerm(search_term) => {
        self.search_term = search_term.clone();
        return if !search_term.is_empty() {
          let wait_duration = Duration::from_millis(300);
          let wait_until = Instant::now() + wait_duration;
          self.search_wait_until = Some(wait_until);
          sleep(wait_duration).perform(|_| SearchCrates).into()
        } else {
          self.search_wait_until = None;
          self.crates = Ok(vec![]);
          Update::empty()
        };
      }
      SearchCrates => if let Some(search_wait_until) = self.search_wait_until {
        if Instant::now() > search_wait_until {
          return client.clone().search_crates(CrateSearch::from_term(self.search_term.clone()))
            .perform(SetCrates).into();
        }
      }
      SetCrates(crates) => {
        if let Err(cause) = &crates {
          error!(?cause, "failed to search for crates");
        }
        self.crates = crates
      },
      FollowCrate(crate_id) => return client.clone().follow_crate(crate_id).perform(ReceiveFollowedCrate).into(),
      ReceiveFollowedCrate(Ok(krate)) => return Update::from_action(Some(krate)),
      ReceiveFollowedCrate(Err(cause)) => error!(?cause, "failed to follow crate"),
      Ignore => {},
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let builder = WidgetBuilder::stack()
      .text_input("Crate search term", &self.search_term).id(self.search_id.clone()).on_input(Message::SetSearchTerm).add();

    let crates = match &self.crates {
      Ok(crates) => {
        let cell_to_element = |row, col| -> Option<Element<Message>> {
          let Some(krate): Option<&Crate> = crates.get(row) else { return None; };
          let element = match col {
            0 => WidgetBuilder::once().add_text(&krate.id),
            1 => WidgetBuilder::once().add_text(&krate.max_version),
            2 => WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string()),
            3 => WidgetBuilder::once().add_text(format!("{}", krate.downloads)),
            4 => WidgetBuilder::once().button("Follow").padding([1.0, 5.0]).positive_style().on_press(|| Message::FollowCrate(krate.id.clone())).add(),
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
