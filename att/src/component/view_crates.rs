use iced::Element;

use crate::app::{Cache, Model};
use crate::widget::builder::WidgetBuilder;

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
    let mut builder = WidgetBuilder::new_heap_with_capacity(model.blessed_crate_ids.len());
    for id in &model.blessed_crate_ids {
      let mut row_builder = WidgetBuilder::default()
        .text(id).width(300).add();
      if let Some(data) = cache.crate_data.get(id) {
        row_builder = row_builder
          .text(&data.max_version).width(150).add()
          .text(data.updated_at.format("%Y-%m-%d").to_string()).width(150).add()
          .text(format!("{}", data.downloads)).width(100).add()
          .button("Remove").destructive_style().padding([1.0, 5.0]).add(|| Message::RemoveCrate(id.clone()))
          .into_row().add()
      } else {
        row_builder = row_builder
          .into_row().add()
      }
      builder = builder.add_element(row_builder.take());
    }
    builder
      .into_column().fill_width().add()
      .into_scrollable().add()
      .take()
  }
}
