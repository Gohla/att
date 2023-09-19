use std::collections::HashMap;
use std::error::Error;

use crates_io_api::{AsyncClient, Crate};
use iced::{Application, Command, Element, Event, event, executor, Length, Renderer, Subscription, Theme, window};
use iced::widget::{self, Button, row, Text};
use serde::{Deserialize, Serialize};

use crate::component::add_crate::{self, AddCrate};
use crate::component::view_crates::{self, ViewCrates};
use crate::widget::{ButtonEx, col};
use crate::widget::modal::Modal;

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
  blessed_crate_ids: Vec<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Cache {
  crate_data: HashMap<String, Crate>
}

pub type SaveFn = Box<dyn FnMut(&Model, &Cache) -> Result<(), Box<dyn Error>> + 'static>;

pub struct Flags {
  pub model: Option<Model>,
  pub cache: Option<Cache>,
  pub save_fn: SaveFn,
  pub crates_io_api: AsyncClient,
}

pub struct App {
  model: Model,
  cache: Cache,

  view_crates: ViewCrates,
  add_crate: AddCrate,
  adding_crate: bool,

  save_fn: SaveFn,
  crates_io_api: AsyncClient,
}

#[derive(Debug)]
pub enum Message {
  ToViewCrates(view_crates::Message),
  ToAddCrate(add_crate::Message),
  OpenAddCrateModal,
  CloseAddCrateModal,
  Exit,
}

impl Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = Theme;
  type Flags = Flags;

  fn new(flags: Flags) -> (Self, Command<Message>) {
    let app = App {
      model: flags.model.unwrap_or_default(),
      cache: flags.cache.unwrap_or_default(),

      view_crates: Default::default(),
      add_crate: Default::default(),
      adding_crate: false,

      save_fn: flags.save_fn,
      crates_io_api: flags.crates_io_api,
    };
    (app, Command::none())
  }
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
      Message::Exit => {
        let _ = (self.save_fn)(&self.model, &self.cache); // TODO: handle error
        return window::close();
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
    let exit_subscription = event::listen_with(|event, _| {
      (event == Event::Window(window::Event::CloseRequested)).then_some(Message::Exit)
    });
    let add_crate_subscription = self.add_crate.subscription(&self.crates_io_api).map(Message::ToAddCrate);
    Subscription::batch([exit_subscription, add_crate_subscription])
  }
}
