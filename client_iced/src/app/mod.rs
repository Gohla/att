use std::error::Error;

use iced::{Element, Event, event, executor, Subscription, Task, window};
use iced_winit::Program;
use tracing::error;

use att_client::{Data, DataRef};
use att_client::auth::{Auth, LoggedIn};
use att_client::http_client::AttHttpClient;
use att_core::users::UserCredentials;
use iced_builder::WidgetBuilder;

use crate::app::follow_crates::FollowCratesComponent;
use crate::perform::PerformExt;
use crate::widget::icon::icon_button;

pub mod search_crates;
pub mod follow_crates;

pub type SaveFn = Box<dyn for<'a> FnMut(DataRef<'a>) -> Result<(), Box<dyn Error>> + 'static>;

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
  dark_mode: bool,
}

#[derive(Debug)]
pub enum Message {
  ToFollowCrates(follow_crates::Message),
  Login(LoggedIn),
  ToggleLightDarkMode,
  Exit(window::Id),
}

impl Program for App {
  type Message = Message;
  type Theme = iced::Theme;
  type Executor = executor::Default;
  type Renderer = iced_renderer::Renderer;
  type Flags = Flags;

  fn new(flags: Flags) -> (Self, Task<Message>) {
    let mut auth = Auth::new(flags.http_client.clone());
    let login_command = auth.login(UserCredentials::default()).perform(Message::Login);

    let app = App {
      save_fn: flags.save_fn,
      follow_crates: FollowCratesComponent::new(flags.http_client, flags.data.follow_crates),
      auth,
      dark_mode: flags.dark_mode,
    };
    let command = Task::batch([login_command]);
    (app, command)
  }

  fn update(&mut self, message: Message) -> Task<Self::Message> {
    use Message::*;
    match message {
      ToFollowCrates(message) => {
        return self.follow_crates.update(message).into_task().map(ToFollowCrates);
      }
      Login(response) => if self.auth.process_logged_in(response).is_ok() {
        return self.follow_crates.request_followed_crates().map(ToFollowCrates);
      }
      ToggleLightDarkMode => { self.dark_mode = !self.dark_mode; }
      Exit(window_id) => {
        let data = DataRef {
          follow_crates: self.follow_crates.state(),
        };
        if let Err(cause) = (self.save_fn)(data) {
          error!(%cause, "failed to save data: {cause:?}");
        }
        return window::close(window_id);
      }
    }
    Task::none()
  }

  fn subscription(&self) -> Subscription<Message> {
    let exit_subscription = event::listen_with::<Message>(|event, _, window_id| {
      if let Event::Window(window::Event::CloseRequested) = event {
        Some(Message::Exit(window_id))
      } else {
        None
      }
    });
    exit_subscription
  }

  fn view(&self, _window_id: window::Id) -> Element<Message> {
    WidgetBuilder::stack()
      .text("All The Things").size(20.0).add()
      .add_space_fill_width()
      .add_element(light_dark_toggle(self.dark_mode, || Message::ToggleLightDarkMode))
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(self.follow_crates.view().map(Message::ToFollowCrates))
      .column().spacing(10.0).padding(10).fill().add()
      .take()
  }

  fn title(&self, _window_id: window::Id) -> String {
    "All The Things".to_string()
  }

  fn theme(&self, _window_id: window::Id) -> iced::Theme {
    match self.dark_mode {
      false => iced::Theme::Light,
      true => iced::Theme::Dark,
    }
  }
}

fn light_dark_toggle<'a, M: 'a>(dark_mode_enabled: bool, on_press: impl Fn() -> M + 'a) -> Element<'a, M> {
  let icon = if dark_mode_enabled { "\u{f5a2}" } else { "\u{f496}" };
  icon_button(icon, on_press)
}
