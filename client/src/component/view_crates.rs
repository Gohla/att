use std::collections::BTreeSet;

use chrono::{DateTime, Duration, Utc};
use crates_io_api::CrateResponse;
use iced::{Command, Element};

use crate::app::{Cache, Model};
use crate::async_util::PerformFutureExt;
use crate::component::{add_crate, Update};
use crate::component::add_crate::AddCrate;
use crate::crates_client::CratesClient;
use crate::widget::builder::WidgetBuilder;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct ViewCrates {
  add_crate: AddCrate,
  adding_crate: bool,
  crates_being_refreshed: BTreeSet<String>,
  crates_client: CratesClient,
}

#[derive(Default, Debug)]
pub enum Message {
  ToAddCrate(add_crate::Message),

  OpenAddCrateModal,
  CloseAddCrateModal,

  RefreshCrate(String),
  RefreshOutdated,
  RefreshAll,

  SetCrateData(Result<CrateResponse, crates_io_api::Error>),
  RemoveCrate(String),

  #[default]
  Ignore,
}

impl ViewCrates {
  pub fn new(crates_client: CratesClient, model: &Model, cache: &Cache) -> (Self, Command<Message>) {
    let mut view_crates = Self {
      add_crate: Default::default(),
      adding_crate: false,
      crates_being_refreshed: Default::default(),
      crates_client,
    };
    let command = view_crates.refresh_outdated_cached_crate_data(model, cache);
    (view_crates, command)
  }

  #[tracing::instrument(skip_all)]
  pub fn update(
    &mut self,
    message: Message,
    model: &mut Model,
    cache: &mut Cache
  ) -> Update<(), Command<Message>> {
    match message {
      Message::ToAddCrate(message) => {
        let (action, command) = self.add_crate.update(message, &self.crates_client).unwrap();
        if let Some(krate) = action {
          model.blessed_crate_ids.insert(krate.id.clone());
          cache.crate_data.insert(krate.id.clone(), (krate, Utc::now()));

          self.add_crate.clear_search_term();
          self.adding_crate = false;
        }
        return command.map(Message::ToAddCrate).into();
      }
      Message::OpenAddCrateModal => {
        self.adding_crate = true;
        return self.add_crate.focus_search_term_input().into();
      }
      Message::CloseAddCrateModal => {
        self.add_crate.clear_search_term();
        self.adding_crate = false;
      }

      Message::RefreshCrate(id) => {
        self.crates_being_refreshed.insert(id.clone());
        return self.crates_client.clone().refresh(id).perform(Message::SetCrateData).into();
      }
      Message::RefreshOutdated => {
        return self.refresh_outdated_cached_crate_data(model, cache).into();
      }
      Message::RefreshAll => {
        return self.refresh_all_cached_crate_data(model, cache).into();
      }

      Message::SetCrateData(Ok(response)) => {
        let id = response.crate_data.id.clone();
        tracing::info!(id, "set crate data");
        self.crates_being_refreshed.remove(&id);
        cache.crate_data.insert(id, (response.crate_data, Utc::now()));
      }
      Message::SetCrateData(Err(cause)) => tracing::error!(?cause, "failed to set crate data"),
      Message::RemoveCrate(id) => {
        model.blessed_crate_ids.remove(&id);
        cache.crate_data.remove(&id);
      }

      Message::Ignore => {}
    }
    Update::default()
  }

  pub fn refresh_outdated_cached_crate_data(&mut self, model: &Model, cache: &Cache) -> Command<Message> {
    let now = Utc::now();
    self.refresh_cached_crate_data(
      model,
      cache,
      |last_refresh| now.signed_duration_since(last_refresh) > Duration::hours(1)
    )
  }

  pub fn refresh_all_cached_crate_data(&mut self, model: &Model, cache: &Cache) -> Command<Message> {
    self.refresh_cached_crate_data(model, cache, |_| true)
  }

  pub fn refresh_cached_crate_data(&mut self, model: &Model, cache: &Cache, should_refresh: impl Fn(&DateTime<Utc>) -> bool) -> Command<Message> {
    let mut commands = Vec::new();
    // Refresh outdated cached crate data.
    for (krate, last_refreshed) in cache.crate_data.values() {
      let id = &krate.id;
      if model.blessed_crate_ids.contains(id) {
        if should_refresh(last_refreshed) {
          commands.push(self.crates_client.clone().refresh(id.clone()).perform(Message::SetCrateData));
          self.crates_being_refreshed.insert(id.clone());
        }
      }
    }
    // Refresh missing cached crate data.
    for id in &model.blessed_crate_ids {
      if !cache.crate_data.contains_key(id) {
        commands.push(self.crates_client.clone().refresh(id.clone()).perform(Message::SetCrateData));
        self.crates_being_refreshed.insert(id.clone());
      }
    }
    Command::batch(commands)
  }


  #[tracing::instrument(skip_all)]
  pub fn view<'a>(&'a self, model: &'a Model, cache: &'a Cache) -> Element<'a, Message> {
    let cell_to_element = |row, col| -> Option<Element<'a, Message>> {
      let Some(id) = model.blessed_crate_ids.iter().nth(row) else { return None; };
      if let Some((data, _)) = cache.crate_data.get(id) {
        match col {
          1 => return Some(WidgetBuilder::once().add_text(&data.max_version)),
          2 => return Some(WidgetBuilder::once().add_text(data.updated_at.format("%Y-%m-%d").to_string())),
          3 => return Some(WidgetBuilder::once().add_text(format!("{}", data.downloads))),
          _ => {}
        }
      }
      let element = match col {
        0 => WidgetBuilder::once().add_text(id),
        4 => WidgetBuilder::once().button("Refresh")
          .padding([1.0, 5.0])
          .on_press(|| Message::RefreshCrate(id.clone()))
          .disabled(self.crates_being_refreshed.contains(id))
          .add(),
        5 => WidgetBuilder::once()
          .button("Remove")
          .destructive_style()
          .padding([1.0, 5.0])
          .on_press(|| Message::RemoveCrate(id.clone()))
          .add(),
        _ => return None,
      };
      Some(element)
    };
    let table = Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(model.blessed_crate_ids.len())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(0.5, "")
      .push(0.5, "")
      .into_element();

    let content = WidgetBuilder::stack()
      .text("Blessed Crates").size(20.0).add()
      .button("Add Crate").on_press(|| Message::OpenAddCrateModal).add()
      .button("Refresh Outdated").on_press(|| Message::RefreshOutdated).disabled(!self.crates_being_refreshed.is_empty()).add()
      .button("Refresh All").on_press(|| Message::RefreshAll).disabled(!self.crates_being_refreshed.is_empty()).add()
      .add_space_fill_width()
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(table)
      .column().spacing(10.0).padding(10).fill().add()
      .take();

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
}
