use std::time::{Duration, Instant};

use iced::{Command, Element};
use iced::widget::text_input;
use tokio::task::AbortHandle;

use att_core::{Crate, Search};

use crate::client::{AttHttpClient, AttHttpClientError};
use crate::component::{Perform, PerformResult, Update};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

#[derive(Debug)]
pub struct FollowCrate {
  search_id: text_input::Id,
  search_term: String,
  search_abort_handle: Option<AbortHandle>,
  crates: Result<Vec<Crate>, AttHttpClientError>,
}

#[derive(Default, Debug)]
pub enum Message {
  SetSearchTerm(String),
  ReceiveCrates(Result<Vec<Crate>, AttHttpClientError>),
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
      search_abort_handle: Default::default(),
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
  #[tracing::instrument(skip_all)]
  pub fn update(&mut self, message: Message, client: &AttHttpClient) -> Update<Option<Crate>, Command<Message>> {
    use Message::*;
    match message {
      SetSearchTerm(search_term) => {
        self.search_term = search_term.clone();
        return if !search_term.is_empty() {
          if let Some(abort_handle) = self.search_abort_handle.take() {
            abort_handle.abort();
          }
          let wait_until = Instant::now() + Duration::from_millis(300);
          let client = client.clone();
          let task = tokio::spawn(async move {
            tokio::time::sleep_until(wait_until.into()).await;
            client.search_crates(Search::from_term(search_term)).await
          });
          self.search_abort_handle = Some(task.abort_handle());
          task.perform_or_default(ReceiveCrates).into()
        } else {
          self.crates = Ok(vec![]);
          if let Some(abort_handle) = self.search_abort_handle.take() {
            abort_handle.abort();
          }
          Update::empty()
        };
      }
      ReceiveCrates(crates) => self.crates = crates,
      FollowCrate(crate_id) => return client.clone().follow_crate(crate_id).perform(ReceiveFollowedCrate).into(),
      ReceiveFollowedCrate(Ok(krate)) => return Update::from_action(Some(krate)),
      ReceiveFollowedCrate(Err(cause)) => tracing::error!(?cause, "failed to follow crate"),
      Ignore => {},
    }
    Update::default()
  }

  #[tracing::instrument(skip(self))]
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
