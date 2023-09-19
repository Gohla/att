use iced::{Element, Length};
use iced::theme;
use iced::widget::{Button, Column, row, Scrollable, Text};

use crate::app::{Cache, Model};
use crate::widget::{ButtonEx, WidgetExt};

#[derive(Default, Debug)]
pub struct ViewCrates;

#[derive(Debug)]
pub enum Message {
  RemoveCrate(String),
}

impl ViewCrates {
  pub fn update(&mut self, message: Message, model: &mut Model, cache: &mut Cache) {
    match message {
      Message::RemoveCrate(id) => {
        model.blessed_crate_ids.remove(&id);
        cache.crate_data.remove(&id);
      }
    }
  }

  pub fn view<'a>(&'a self, model: &'a Model, cache: &'a Cache) -> Element<'_, Message> {
    let mut crate_rows = Vec::new();
    for id in &model.blessed_crate_ids {
      let id_text = Text::new(id).width(300);
      let element = if let Some(data) = cache.crate_data.get(id) {
        let remove_button = Button::new(Text::new("Remove"))
          .style(theme::Button::Destructive)
          .padding([1.0, 5.0, 1.0, 5.0])
          .on_press_into_element(|| Message::RemoveCrate(id.clone()));
        let row = row![
          id_text,
          Text::new(&data.max_version).width(150),
          Text::new(data.updated_at.format("%Y-%m-%d").to_string()).width(150),
          Text::new(format!("{}", data.downloads)).width(100),
          remove_button,
        ];
        row.into_element()
      } else {
        id_text.into_element()
      };
      crate_rows.push(element)
    }
    let column = Column::with_children(crate_rows)
      .width(Length::Fill);
    Scrollable::new(column)
      .into_element()
  }
}
