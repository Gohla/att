use std::error::Error;

use iced::{Command, Element, Event, event, executor, Subscription, Theme, window};
use iced::window::Id;
use tracing::error;

use att_client::auth::{Auth, LoggedIn};
use att_client::Data;
use att_client::http_client::AttHttpClient;
use att_core::users::UserCredentials;

use crate::app::follow_crates::FollowCratesComponent;
use crate::update::Perform;
use crate::widget::builder::WidgetBuilder;
use crate::widget::dark_light_toggle::light_dark_toggle;

pub mod search_crates;
pub mod follow_crates;

pub type SaveFn = Box<dyn FnMut(&Data) -> Result<(), Box<dyn Error>> + 'static>;

pub struct Flags {
  pub http_client: AttHttpClient,
  pub save_fn: SaveFn,
  pub data: Data,
  pub dark_mode: bool,
}

pub struct App {
  save_fn: SaveFn,
  follow_crates: FollowCratesComponent,
  auth: Auth,
  data: Data,
  dark_mode: bool,
}

#[derive(Debug)]
pub enum Message {
  ToFollowCrates(follow_crates::Message),
  Login(LoggedIn),
  ToggleLightDarkMode,
  Exit(Id),
}

impl iced::Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Theme = Theme;
  type Flags = Flags;

  fn new(flags: Flags) -> (Self, Command<Message>) {
    let mut auth = Auth::new(flags.http_client.clone());
    let login_command = auth.login(UserCredentials::default()).perform(Message::Login);

    let app = App {
      save_fn: flags.save_fn,
      follow_crates: FollowCratesComponent::new(flags.http_client),
      auth,
      data: flags.data,
      dark_mode: flags.dark_mode,
    };
    let command = Command::batch([login_command]);
    (app, command)
  }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    use Message::*;
    match message {
      ToFollowCrates(message) => {
        return self.follow_crates.update(message, self.data.crates_mut()).into_command().map(ToFollowCrates);
      }
      Login(response) => if self.auth.process_logged_in(response).is_ok() {
        return self.follow_crates.request_followed_crates().map(ToFollowCrates);
      }
      ToggleLightDarkMode => { self.dark_mode = !self.dark_mode; }
      Exit(window_id) => {
        if let Err(cause) = (self.save_fn)(&self.data) {
          error!(%cause, "failed to save data: {cause:?}");
        }
        return window::close(window_id);
      }
    }
    Command::none()
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

  fn view(&self) -> Element<Message> {
    WidgetBuilder::stack()
      .text("All The Things").size(20.0).add()
      .add_space_fill_width()
      .add_element(light_dark_toggle(self.dark_mode, || Message::ToggleLightDarkMode))
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(self.follow_crates.view(self.data.crates()).map(Message::ToFollowCrates))
      .column().spacing(10.0).padding(10).fill().add()
      .take()
  }

  fn title(&self) -> String { "All The Things".to_string() }

  fn theme(&self) -> Theme {
    match self.dark_mode {
      false => Theme::Light,
      true => Theme::Dark,
    }
  }
}
