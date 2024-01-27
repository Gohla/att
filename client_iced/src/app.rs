use std::error::Error;

use iced::{Command, Element, Event, event, executor, Renderer, Subscription, Theme, window};
use iced::window::Id;
use tracing::error;

use att_client::{AttClient, Data, Login, ViewData};
use att_core::users::UserCredentials;

use crate::component::Perform;
use crate::component::view_followed_crates::{self, ViewFollowedCrates};
use crate::widget::builder::WidgetBuilder;
use crate::widget::dark_light_toggle::light_dark_toggle;

pub type SaveFn = Box<dyn FnMut(&Data) -> Result<(), Box<dyn Error>> + 'static>;

pub struct Flags {
  pub data: Data,
  pub dark_mode: bool,
  pub client: AttClient,
  pub save_fn: SaveFn,
}

pub struct App {
  view_data: ViewData,
  data: Data,

  dark_mode: bool,

  view_followed_crates: ViewFollowedCrates,

  save_fn: SaveFn,
}

#[derive(Debug)]
pub enum Message {
  ToViewFollowedCrates(view_followed_crates::Message),

  Login(Login),

  ToggleLightDarkMode,

  Exit(Id),
}

impl iced::Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = Theme;
  type Flags = Flags;

  fn new(flags: Flags) -> (Self, Command<Message>) {
    let mut view_data = ViewData::default();
    let data = flags.data;

    let login_command = flags.client.clone().login(view_data.app_mut(), UserCredentials::default())
      .perform(Message::Login);

    let app = App {
      view_data,
      data,
      dark_mode: flags.dark_mode,
      view_followed_crates: ViewFollowedCrates::new(flags.client),
      save_fn: flags.save_fn,
    };
    let command = Command::batch([login_command]);
    (app, command)
  }
  fn title(&self) -> String { "All The Things".to_string() }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    match message {
      Message::ToViewFollowedCrates(message) => {
        return self.view_followed_crates.update(message, self.view_data.crates_mut(), self.data.crates_mut())
          .into_command()
          .map(|m| Message::ToViewFollowedCrates(m));
      }

      Message::Login(login) => {
        if login.apply(self.view_data.app_mut()).is_ok() {
          return self.view_followed_crates.request_followed_crates(self.view_data.crates_mut()).map(Message::ToViewFollowedCrates);
        }
      }

      Message::ToggleLightDarkMode => {
        self.dark_mode = !self.dark_mode;
      }

      Message::Exit(window_id) => {
        if let Err(cause) = (self.save_fn)(&self.data) {
          error!(%cause, "failed to save data: {cause:?}");
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
      .add_element(self.view_followed_crates.view(self.data.crates(), self.view_data.crates()).map(Message::ToViewFollowedCrates))
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
