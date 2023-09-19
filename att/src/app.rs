use crates_io_api::AsyncClient;
use iced::{Application, Command, Element, executor, Length, Renderer, Subscription, Theme};
use iced::widget::{self, Button, row, Text};

use crate::component::add_crate::{self, AddCrate};
use crate::component::view_crates::{self, ViewCrates};
use crate::widget::{ButtonEx, col};
use crate::widget::modal::Modal;

pub struct App {
  crates_io_api: AsyncClient,

  view_crates: ViewCrates,
  add_crate: AddCrate,
  adding_crate: bool,
}

impl App {
  pub fn new(crates_io_api: AsyncClient) -> Self {
    Self {
      crates_io_api,
      view_crates: ViewCrates::default(),
      add_crate: AddCrate::default(),
      adding_crate: false,
    }
  }
}

#[derive(Debug)]
pub enum Message {
  ToViewCrates(view_crates::Message),
  ToAddCrate(add_crate::Message),
  OpenAddCrateModal,
  CloseAddCrateModal,
}

impl Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = Theme;
  type Flags = App;

  fn new(flags: Self) -> (Self, Command<Message>) { (flags, Command::none()) }
  fn title(&self) -> String { "All The Things".to_string() }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    match message {
      Message::ToViewCrates(message) => {
        self.view_crates.update(message);
      }
      Message::ToAddCrate(message) => {
        if let Some(krate) = self.add_crate.update(message).into_action() {
          self.add_crate.clear_search_term();
          self.view_crates.add_crate(krate);
          self.adding_crate = false;
        }
      }
      Message::OpenAddCrateModal => {
        self.adding_crate = true;
        return widget::focus_next();
      }
      Message::CloseAddCrateModal => {
        self.add_crate.clear_search_term();
        self.adding_crate = false;
      }
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Message, Renderer<Theme>> {
    let header = {
      let text = Text::new("Blessed Crates")
        .size(20.0);
      let add_crate_button = Button::new("Add Crate")
        .on_press_into_element(|| Message::OpenAddCrateModal);
      row![text, add_crate_button]
        .spacing(10.0)
        .width(Length::Fill)
    };
    let view_crates = self.view_crates
      .view()
      .map(Message::ToViewCrates);
    let content = col![header, view_crates]
      .spacing(10.0)
      .padding(10)
      .width(Length::Fill)
      .height(Length::Fill);

    if self.adding_crate {
      let overlay = self.add_crate
        .view()
        .map(Message::ToAddCrate);
      let modal = Modal::new(overlay, content)
        .on_close_modal(|| Message::CloseAddCrateModal);
      modal.into()
    } else {
      content.into()
    }
  }

  fn theme(&self) -> Theme { Theme::Light }

  fn subscription(&self) -> Subscription<Message> {
    self.add_crate.subscription(&self.crates_io_api).map(Message::ToAddCrate)
  }
}
