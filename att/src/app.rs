use std::collections::{BTreeSet, HashMap};
use std::error::Error;

use crates_io_api::{AsyncClient, Crate};
use iced::{Alignment, Application, Command, Element, Event, event, executor, Length, Renderer, Subscription, Theme, window};
use iced::widget::Rule;
use serde::{Deserialize, Serialize};

use crate::component::add_crate::{self, AddCrate};
use crate::component::view_crates::{self, ViewCrates};
use crate::widget::{col, load_icon_font_command};
use crate::widget::builder::Builder;
use crate::widget::dark_light_toggle::light_dark_toggle;
use crate::widget::modal::Modal;

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
  pub blessed_crate_ids: BTreeSet<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Cache {
  pub crate_data: HashMap<String, Crate>
}

pub type SaveFn = Box<dyn FnMut(&Model, &Cache) -> Result<(), Box<dyn Error>> + 'static>;

pub struct Flags {
  pub model: Option<Model>,
  pub cache: Option<Cache>,
  pub dark_mode: bool,
  pub save_fn: SaveFn,
  pub crates_io_api: AsyncClient,
}

pub struct App {
  model: Model,
  cache: Cache,

  view_crates: ViewCrates,
  add_crate: AddCrate,
  adding_crate: bool,
  dark_mode: bool,

  save_fn: SaveFn,
  crates_io_api: AsyncClient,
}

#[derive(Debug)]
pub enum Message {
  ToViewCrates(view_crates::Message),
  ToAddCrate(add_crate::Message),

  OpenAddCrateModal,
  CloseAddCrateModal,
  ToggleLightDarkMode,

  FontLoaded(Result<(), iced::font::Error>),
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
      dark_mode: flags.dark_mode,

      save_fn: flags.save_fn,
      crates_io_api: flags.crates_io_api,
    };
    (app, load_icon_font_command(Message::FontLoaded))
  }
  fn title(&self) -> String { "All The Things".to_string() }

  fn update(&mut self, message: Message) -> Command<Self::Message> {
    match message {
      Message::ToViewCrates(message) => {
        self.view_crates.update(message, &mut self.model, &mut self.cache);
      }
      Message::ToAddCrate(message) => {
        if let Some(krate) = self.add_crate.update(message).into_action() {
          self.model.blessed_crate_ids.insert(krate.id.clone());
          self.cache.crate_data.insert(krate.id.clone(), krate);

          self.add_crate.clear_search_term();
          self.adding_crate = false;
        }
      }

      Message::OpenAddCrateModal => {
        self.adding_crate = true;
        return self.add_crate.focus_search_term_input();
      }
      Message::CloseAddCrateModal => {
        self.add_crate.clear_search_term();
        self.adding_crate = false;
      }
      Message::ToggleLightDarkMode => {
        self.dark_mode = !self.dark_mode;
      }

      Message::FontLoaded(_) => {},
      Message::Exit => {
        let _ = (self.save_fn)(&self.model, &self.cache); // TODO: handle error
        return window::close();
      }
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Message, Renderer<Theme>> {
    let header = Builder::default()
      .text("Blessed Crates").size(20.0).done()
      .button("Add Crate").done(|| Message::OpenAddCrateModal)
      .space().fill_width().done()
      .element(light_dark_toggle(self.dark_mode, || Message::ToggleLightDarkMode))
      .into_row().spacing(10.0).align_items(Alignment::Center).width(Length::Fill).done()
      .take();
    let rule = Rule::horizontal(1.0);
    let view_crates = self.view_crates
      .view(&self.model, &self.cache)
      .map(Message::ToViewCrates);
    let content = col![header, rule, view_crates]
      .spacing(10.0)
      .padding(10)
      .width(Length::Fill)
      .height(Length::Fill);

    if self.adding_crate {
      let overlay = self.add_crate
        .view()
        .map(Message::ToAddCrate);
      let modal = Modal::with_container(overlay, content)
        .on_close_modal(|| Message::CloseAddCrateModal);
      modal.into()
    } else {
      content.into()
    }
  }

  fn theme(&self) -> Theme {
    match self.dark_mode {
      false => Theme::Light,
      true => Theme::Dark,
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    let exit_subscription = event::listen_with(|event, _| {
      (event == Event::Window(window::Event::CloseRequested)).then_some(Message::Exit)
    });
    let add_crate_subscription = self.add_crate.subscription(&self.crates_io_api).map(Message::ToAddCrate);
    Subscription::batch([exit_subscription, add_crate_subscription])
  }
}
