use std::collections::HashMap;

use crates_io_api::Crate;
use iced::{Element, Length};
use iced::theme;
use iced::widget::{Button, Column, row, Scrollable, Text};

use crate::widget::{ButtonEx, WidgetExt};

#[derive(Default, Debug)]
pub struct ViewCrates {
  crates: HashMap<String, Crate>,
}

#[derive(Debug)]
pub enum Message {
  RemoveCrate(String),
}

impl ViewCrates {
  pub fn add_crate(&mut self, krate: Crate) {
    self.crates.insert(krate.id.clone(), krate);
  }
}

impl ViewCrates {
  pub fn update(&mut self, message: Message) {
    match message {
      Message::RemoveCrate(id) => {
        self.crates.remove(&id);
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let mut crate_rows = Vec::new();
    for krate in self.crates.values() {
      let remove_button = Button::new(Text::new("Remove"))
        .style(theme::Button::Destructive)
        .padding([1.0, 5.0, 1.0, 5.0])
        .on_press_into_element(|| Message::RemoveCrate(krate.id.clone()));
      let row = row![
        Text::new(&krate.id).width(300),
        Text::new(&krate.max_version).width(150),
        Text::new(krate.updated_at.format("%Y-%m-%d").to_string()).width(150),
        Text::new(format!("{}", krate.downloads)).width(100),
        remove_button,
      ];
      crate_rows.push(row.into())
    }
    let column = Column::with_children(crate_rows)
      .width(Length::Fill);
    Scrollable::new(column)
      .into_element()
  }
}
