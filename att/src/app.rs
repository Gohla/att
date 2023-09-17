use crates_io_api::AsyncClient;
use iced::{Application, Command, Element, executor, Renderer, Subscription};
use iced::widget::Container;
use iced_core::Length;

use crate::{add_crate, col};
use crate::add_crate::AddCrate;
use crate::util::WidgetExt;

pub type AppTheme = iced::Theme;
pub type AppRenderer = Renderer<AppTheme>;

pub struct App {
  crates_io_api: AsyncClient,
  add_crate: Option<AddCrate>,
}

impl App {
  pub fn new(crates_io_api: AsyncClient) -> Self {
    Self { crates_io_api, add_crate: Some(AddCrate::default()) }
  }
}

#[derive(Debug)]
pub enum Message {
  ToAddCrate(add_crate::Message),
}

impl Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = AppTheme;
  type Flags = App;

  fn new(flags: Self) -> (Self, Command<Message>) { (flags, Command::none()) }
  fn title(&self) -> String { "All The Things".to_string() }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    match message {
      Message::ToAddCrate(message) => {
        if let Some(add_crate) = &mut self.add_crate {
          add_crate.update(message).inspect_action(|krate| {
            println!("Add crate: {:?}", krate);
          });
        }
      }
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Message, AppRenderer> {
    let content = if let Some(add_crate) = &self.add_crate {
      add_crate
        .view()
        .map_into_element(|m| Message::ToAddCrate(m))
    } else {
      col![].into_element()
    };
    Container::new(content)
      .width(Length::Fill)
      .padding(40)
      .center_x()
      .into()
  }

  fn subscription(&self) -> Subscription<Message> {
    if let Some(add_crate) = &self.add_crate {
      add_crate.subscription(&self.crates_io_api)
        .map(|m| Message::ToAddCrate(m))
    } else {
      Subscription::none()
    }
  }
}
