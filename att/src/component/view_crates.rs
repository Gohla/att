use iced::{Color, Element};

use crate::app::{Cache, Model};
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::TableBuilder;

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
    let table = TableBuilder::new(model.blessed_crate_ids.len(), |row_index, column_index| -> Element<'a, Message>{
      let Some(id) = model.blessed_crate_ids.iter().nth(row_index) else {
        return WidgetBuilder::default().add_space_fill_width().take()
      };
      let Some(data) = cache.crate_data.get(id) else {
        return WidgetBuilder::default().add_space_fill_width().take()
      };
      match column_index {
        0 => WidgetBuilder::default().add_text(id).take(),
        1 => WidgetBuilder::default().add_text(&data.max_version).take(),
        2 => WidgetBuilder::default().add_text(data.updated_at.format("%Y-%m-%d").to_string()).take(),
        3 => WidgetBuilder::default().add_text(format!("{}", data.downloads)).take(),
        4 => WidgetBuilder::default().button("Remove").destructive_style().padding([1.0, 5.0]).add(|| Message::RemoveCrate(id.clone())).take(),
        _ => WidgetBuilder::default().add_space_fill_width().take()
      }
    })
      .push_column(2, "Name")
      .push_column(1, "Latest Version")
      .push_column(1, "Updated at")
      .push_column(1, "Downloads")
      .push_column(1, "")
      .build();
    Element::new(table)
      .explain(Color::BLACK)
    // let mut builder = WidgetBuilder::new_heap_with_capacity(model.blessed_crate_ids.len());
    // for id in &model.blessed_crate_ids {
    //   let mut row_builder = WidgetBuilder::default()
    //     .text(id).width(300).add();
    //   if let Some(data) = cache.crate_data.get(id) {
    //     row_builder = row_builder
    //       .text(&data.max_version).width(150).add()
    //       .text(data.updated_at.format("%Y-%m-%d").to_string()).width(150).add()
    //       .text(format!("{}", data.downloads)).width(100).add()
    //       .button("Remove").destructive_style().padding([1.0, 5.0]).add(|| Message::RemoveCrate(id.clone()))
    //       .into_row().add()
    //   } else {
    //     row_builder = row_builder
    //       .into_row().add()
    //   }
    //   builder = builder.add_element(row_builder.take());
    // }
    // builder
    //   .into_column().fill_width().add()
    //   .into_scrollable().add()
    //   .take()
  }
}
