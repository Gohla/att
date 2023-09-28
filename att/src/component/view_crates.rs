use iced::{Color, Element};

use crate::app::{Cache, Model};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

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

  pub fn view<'a>(&'a self, model: &'a Model, cache: &'a Cache) -> Element<'a, Message> {
    let cell_to_element = |row, col| -> Element<'a, Message>{
      let Some(id) = model.blessed_crate_ids.iter().nth(row) else {
        return WidgetBuilder::default().add_space_fill_width().take()
      };
      let Some(data) = cache.crate_data.get(id) else {
        return WidgetBuilder::default().add_space_fill_width().take()
      };
      match col {
        0 => WidgetBuilder::default().add_text(id).take(),
        1 => WidgetBuilder::default().add_text(&data.max_version).take(),
        2 => WidgetBuilder::default().add_text(data.updated_at.format("%Y-%m-%d").to_string()).take(),
        3 => WidgetBuilder::default().add_text(format!("{}", data.downloads)).take(),
        4 => WidgetBuilder::default().button("Remove").destructive_style().padding([1.0, 5.0]).add(|| Message::RemoveCrate(id.clone())).take(),
        _ => WidgetBuilder::default().add_space_fill_width().take()
      }
    };
    Table::with_capacity(5, cell_to_element)
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(1, "")
      .body_row_count(model.blessed_crate_ids.len())
      .into_element()
      .explain(Color::WHITE)
  }
}
