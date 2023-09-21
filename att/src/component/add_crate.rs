use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, Crate, CratesPage, CratesQuery, Sort};
use iced::{Command, Element, futures, Length, Subscription, theme};
use iced::widget::{Button, Column, row, Scrollable, Text, text_input};

use crate::component::Update;
use crate::widget::{ButtonEx, col, WidgetExt};

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

  pub fn view(&self) -> Element<'_, Message> {
    let search_term_input = text_input::TextInput::new("Crate search term", &self.search_term)
      .id(self.search_id.clone())
      .on_input(|s| s)
      .map_into_element(Message::SetSearchTerm);

    let crates = match &self.crates {
      Some(Ok(crates)) => {
        let mut crate_rows = Vec::new();
        for krate in &crates.crates {
          let add_button = Button::new(Text::new("Add"))
            .style(theme::Button::Positive)
            .padding([0.0, 5.0, 0.0, 5.0])
            .on_press_into_element(|| Message::AddCrate(krate.clone()));
          let row = row![
            Text::new(&krate.id).width(300),
            Text::new(&krate.max_version).width(150),
            Text::new(krate.updated_at.format("%Y-%m-%d").to_string()).width(150),
            Text::new(format!("{}", krate.downloads)).width(100),
            add_button,
          ];
          crate_rows.push(row.into())
        }
        let column = Column::with_children(crate_rows)
          .spacing(2.0)
          .width(Length::Fill);
        Scrollable::new(column)
          .into_element()
      }
      Some(Err(e)) => Text::new(format!("{:?}", e)).into_element(),
      _ => col![].into_element()
    };

    col![search_term_input, crates]
      .spacing(20)
      .width(800)
      .height(600)
      .into()
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
