use std::error::Error;
use std::time::{Duration, Instant};

use crates_io_api::{AsyncClient, CratesPage, CratesQuery, Sort};
use iced::{Application, Command, Element, executor, futures, Renderer, Settings, Subscription, subscription};
use iced::widget::{Column, Container, Row, Scrollable, Text, TextInput};
use iced_core::{Length, Widget};

fn main() -> Result<(), Box<dyn Error>> {
  let crates_io_api = AsyncClient::new("Gohla (https://github.com/Gohla)", Duration::from_secs(1))?;
  let app = App {
    crates_io_api,
    search_text: String::new(),
    next_search: None,
    crates: None,
  };
  App::run(Settings::with_flags(app))?;
  Ok(())
}

pub struct App {
  crates_io_api: AsyncClient,
  search_text: String,
  next_search: Option<Instant>,
  crates: Option<CratesPage>,
}

#[derive(Debug)]
pub enum Msg {
  SearchTextChange(String),
  CratesChange(CratesPage),
  Error,
}

impl Application for App {
  type Executor = executor::Default;
  type Message = Msg;
  type Theme = iced::Theme;
  type Flags = App;

  fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
    (flags, Command::none())
  }
  fn title(&self) -> String {
    "All The Things".to_string()
  }

  fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
      Msg::SearchTextChange(s) => {
        self.search_text = s;
        if !self.search_text.is_empty() {
          self.next_search = Some(Instant::now() + Duration::from_millis(200));
        } else {
          self.next_search = None;
          self.crates = None;
        }
      }
      Msg::CratesChange(crates) => self.crates = Some(crates),
      Msg::Error => {}
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
    let search_text_input = TextInput::new("Search for crate", &self.search_text)
      .on_input(|s| s)
      .map_into_element(|s| Msg::SearchTextChange(s));

    let mut crate_elements = Vec::new();
    if let Some(crates) = &self.crates {
      for c in &crates.crates {
        let row = Row::new()
          .push(Text::new(&c.id).width(350))
          .push(Text::new(&c.max_version).width(150))
          .push(Text::new(c.updated_at.format("%Y-%m-%d").to_string()).width(150))
          .push(Text::new(format!("{}", c.downloads)).width(150))
          ;
        crate_elements.push(row.into())
      }
    }
    let crates = Scrollable::new(Column::with_children(crate_elements).width(Length::Fill));

    let content = Column::with_children(vec![
      search_text_input,
      crates.into(),
    ]);
    let content = content
      .spacing(20)
      .max_width(800)
      ;
    Container::new(content)
      .width(Length::Fill)
      .padding(40)
      .center_x()
      .into()
  }

  fn subscription(&self) -> Subscription<Self::Message> {
    if let Some(next_search) = self.next_search {
      let search_text = self.search_text.clone();
      let crates_io_api = self.crates_io_api.clone();
      let stream = futures::stream::once(async move {
        tokio::time::sleep_until(next_search.into()).await;
        let query = CratesQuery::builder()
          .search(search_text)
          .sort(Sort::Relevance)
          .build();
        let Ok(response) = crates_io_api.crates(query).await else {
          return Msg::Error;
        };
        Msg::CratesChange(response)
      });
      subscription::run_with_id(next_search, stream)
    } else {
      Subscription::none()
    }
  }
}

pub trait WidgetExt<'a, M, R> {
  fn into_element(self) -> Element<'a, M, R>;
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R>;
}

impl<'a, M: 'a, R: iced_core::Renderer + 'a, W: Widget<M, R> + 'a> WidgetExt<'a, M, R> for W {
  #[inline]
  fn into_element(self) -> Element<'a, M, R> {
    Element::new(self)
  }
  #[inline]
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R> {
    self.into_element().map(f)
  }
}
