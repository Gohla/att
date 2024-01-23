use std::collections::BTreeMap;
use std::error::Error;

use iced::{Command, Element, Event, event, executor, Renderer, Subscription, Theme, window};
use iced::window::Id;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use att_client::http_client::{AttHttpClient, AttHttpClientError};
use att_core::crates::Crate;
use att_core::users::UserCredentials;

use crate::component::Perform;
use crate::component::view_crates::{self, ViewCrates};
use crate::widget::builder::WidgetBuilder;
use crate::widget::dark_light_toggle::light_dark_toggle;

#[derive(Default, Serialize, Deserialize)]
pub struct Data {}

#[derive(Default, Serialize, Deserialize)]
pub struct Cache {
  pub id_to_crate: BTreeMap<String, Crate>,
}

pub type SaveFn = Box<dyn FnMut(&Data, &Cache) -> Result<(), Box<dyn Error>> + 'static>;

pub struct Flags {
  pub data: Option<Data>,
  pub cache: Option<Cache>,
  pub save_fn: SaveFn,
  pub client: AttHttpClient,
  pub dark_mode: bool,
}

pub struct App {
  data: Data,
  cache: Cache,
  save_fn: SaveFn,

  view_crates: ViewCrates,
  dark_mode: bool,
}

#[derive(Debug)]
pub enum Message {
  ToViewCrates(view_crates::Message),

  LoginResponse(Result<(), AttHttpClientError>),

  ToggleLightDarkMode,

  Exit(Id),
}

impl iced::Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = Theme;
  type Flags = Flags;

  fn new(flags: Flags) -> (Self, Command<Message>) {
    let data = flags.data.unwrap_or_default();
    let cache = flags.cache.unwrap_or_default();

    let login_command = flags.client.clone().login(UserCredentials::default())
      .perform(Message::LoginResponse);

    let view_crates = ViewCrates::new(flags.client.clone(), &cache);

    let app = App {
      data,
      cache,
      save_fn: flags.save_fn,

      view_crates,
      dark_mode: flags.dark_mode,
    };

    let command = Command::batch([login_command]);

    (app, command)
  }
  fn title(&self) -> String { "All The Things".to_string() }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    match message {
      Message::ToViewCrates(message) => {
        return self.view_crates.update(message)
          .into_command()
          .map(|m| Message::ToViewCrates(m));
      }

      Message::LoginResponse(result) => {
        match result {
          Ok(_) => {
            info!("logged in");
            return self.view_crates.request_followed_crates().map(Message::ToViewCrates);
          }
          Err(cause) => error!(%cause, "failed to login: {cause:?}"),
        }
      }

      Message::ToggleLightDarkMode => {
        self.dark_mode = !self.dark_mode;
      }

      Message::Exit(window_id) => {
        self.view_crates.cache(&mut self.cache);
        if let Err(cause) = (self.save_fn)(&self.data, &self.cache) {
          error!(%cause, "failed to save data and cache: {cause:?}");
        }
        return window::close(window_id);
      }
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Message, Renderer<Theme>> {
    let content = WidgetBuilder::stack()
      .text("All The Things").size(20.0).add()
      .add_space_fill_width()
      .add_element(light_dark_toggle(self.dark_mode, || Message::ToggleLightDarkMode))
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(self.view_crates.view().map(Message::ToViewCrates))
      .column().spacing(10.0).padding(10).fill().add()
      .take();

    content.into()
  }

  fn theme(&self) -> Theme {
    match self.dark_mode {
      false => Theme::Light,
      true => Theme::Dark,
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    let exit_subscription = event::listen_with(|event, _| {
      if let Event::Window(id, window::Event::CloseRequested) = event {
        Some(Message::Exit(id))
      } else {
        None
      }
    });
    exit_subscription
  }
}
