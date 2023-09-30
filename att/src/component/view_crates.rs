use crates_io_api::CrateResponse;
use iced::{Command, Element};

use crate::app::{Cache, Model};
use crate::component::Update;
use crate::crates_client::CratesClient;
use crate::widget::builder::WidgetBuilder;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

#[derive(Default, Debug)]
pub struct ViewCrates;

#[derive(Debug)]
pub enum Message {
  RequestCrateUpdate(String),
  ReceiveCrateUpdate(Result<CrateResponse, crates_io_api::Error>),
  RemoveCrate(String),
  Ignore,
}

impl ViewCrates {
  pub fn update(&mut self, message: Message, crates_client: &CratesClient, model: &mut Model, cache: &mut Cache) -> Update<(), Command<Message>> {
    match message {
      Message::RequestCrateUpdate(crate_name) => {
        // TODO: make Ignore message default and add convenience to crates_client that maps to default on async error, and then maps the message
        return Update::perform(crates_client.clone().update(crate_name), |r| r.map_or(Message::Ignore, |r| Message::ReceiveCrateUpdate(r)))
      }
      Message::ReceiveCrateUpdate(r) => {
        match r {
          Ok(r) => {
            println!("Updated data for crate: {}", r.crate_data.id);
            cache.crate_data.insert(r.crate_data.id.clone(), r.crate_data);
          }
          Err(e) => println!("Failed to retrieve crate data: {:?}", e),
        }
      }
      Message::RemoveCrate(id) => {
        model.blessed_crate_ids.remove(&id);
        cache.crate_data.remove(&id);
      }
      Message::Ignore => {}
    }
    Update::empty()
  }

  pub fn view<'a>(&'a self, model: &'a Model, cache: &'a Cache) -> Element<'a, Message> {
    let cell_to_element = |row, col| -> Option<Element<'a, Message>> {
      let Some(id) = model.blessed_crate_ids.iter().nth(row) else { return None; };
      let Some(data) = cache.crate_data.get(id) else { return None; };
      let element = match col {
        0 => WidgetBuilder::once().add_text(id),
        1 => WidgetBuilder::once().add_text(&data.max_version),
        2 => WidgetBuilder::once().add_text(data.updated_at.format("%Y-%m-%d").to_string()),
        3 => WidgetBuilder::once().add_text(format!("{}", data.downloads)),
        4 => WidgetBuilder::once().button("Update").primary_style().padding([1.0, 5.0]).on_press(|| Message::RequestCrateUpdate(id.clone())).add(),
        5 => WidgetBuilder::once().button("Remove").destructive_style().padding([1.0, 5.0]).on_press(|| Message::RemoveCrate(id.clone())).add(),
        _ => return None,
      };
      Some(element)
    };
    Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(model.blessed_crate_ids.len())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(0.5, "")
      .push(0.5, "")
      .into_element()
  }
}
