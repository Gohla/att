use crates_io_api::AsyncClient;
use iced::{Alignment, Application, Command, Element, executor, Length, Renderer, Subscription};
use iced::widget::{Container, row, Space, Text};

use crate::{add_crate, col};
use crate::add_crate::AddCrate;
use crate::modal::Modal;

pub type AppTheme = iced::Theme;
pub type AppRenderer = Renderer<AppTheme>;

pub struct App {
  crates_io_api: AsyncClient,
  add_crate: AddCrate,
}

impl App {
  pub fn new(crates_io_api: AsyncClient) -> Self {
    Self {
      crates_io_api,
      add_crate: AddCrate::default(),
    }
  }
}

#[derive(Debug)]
pub enum Message {
  ToAddCrate(add_crate::Message),
  ShowModal,
  CloseModal,
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
        if let Some(krate) = self.add_crate.update(message).into_action() {
          println!("Add crate: {:?}", krate);
        }
      }
      Message::ShowModal => dbg!(),
      Message::CloseModal => dbg!(),
    }
    Command::none()
  }

  fn view(&self) -> Element<'_, Message, AppRenderer> {
    let content = Container::new(col![
        row![Text::new("Top Left"), Space::with_width(Length::Fill), Text::new("Top Right")]
          .align_items(Alignment::Start)
          .height(Length::Fill),
        row![Text::new("Bottom Left"), Space::with_width(Length::Fill), Text::new("Bottom Right")]
          .align_items(Alignment::End)
          .height(Length::Fill)
      ].height(Length::Fill))
      .padding(10)
      .width(Length::Fill)
      .height(Length::Fill);

    let add_crate = self.add_crate
      .view()
      .map(Message::ToAddCrate);

    let modal = Modal::new(content, add_crate)
      .on_press_parent_area(|| Message::CloseModal);
    modal.into()
  }

  fn subscription(&self) -> Subscription<Message> {
    self.add_crate.subscription(&self.crates_io_api).map(Message::ToAddCrate)
  }
}
