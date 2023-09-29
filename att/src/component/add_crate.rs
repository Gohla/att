use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, Crate, CratesPage, CratesQuery, Sort};
use iced::{Command, Element, futures, Subscription};
use iced::widget::text_input;

use crate::component::Update;
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

/// Search for a crate on crates.io and add it.
#[derive(Debug)]
pub struct AddCrate {
  wait_before_searching: Duration,
  search_id: text_input::Id,
  search_term: String,
  next_search_time: Option<Instant>,
  crates: Option<Result<CratesPage, crates_io_api::Error>>,
}

#[derive(Debug)]
pub enum Message {
  SetSearchTerm(String),
  SetCrates(Result<CratesPage, crates_io_api::Error>),
  AddCrate(Crate),
}

impl Default for AddCrate {
  fn default() -> Self {
    Self {
      wait_before_searching: Duration::from_millis(200),
      search_id: text_input::Id::unique(),
      search_term: String::new(),
      next_search_time: None,
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
    self.next_search_time = None;
    self.crates = None;
  }
}

impl AddCrate {
  pub fn update(&mut self, message: Message) -> Update<Option<Crate>> {
    match message {
      Message::SetSearchTerm(s) => {
        self.search_term = s;
        if !self.search_term.is_empty() {
          self.next_search_time = Some(Instant::now() + self.wait_before_searching);
        } else {
          self.next_search_time = None;
          self.crates = None;
        }
      }
      Message::SetCrates(crates) => self.crates = Some(crates),
      Message::AddCrate(krate) => {
        return Update::from_action(krate)
      },
    }
    Update::default()
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

  pub fn subscription(&self, crates_io_api: &AsyncClient) -> Subscription<Message> {
    let Some(next_search) = self.next_search_time else {
      return Subscription::none();
    };
    let search_term = self.search_term.clone();
    let crates_io_api = crates_io_api.clone();
    let stream = futures::stream::once(async move {
      tokio::time::sleep_until(next_search.into()).await;
      let query = CratesQuery::builder()
        .search(search_term)
        .sort(Sort::Relevance)
        .build();
      let response = crates_io_api.crates(query).await;
      Message::SetCrates(response)
    });
    iced::subscription::run_with_id(next_search, stream)
  }
}
