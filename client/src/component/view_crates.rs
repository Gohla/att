use std::collections::{BTreeSet, HashMap};

use iced::{Command, Element};

use att_core::{Crate, Search};

use crate::app::{Cache, Data};
use crate::async_util::PerformFutureExt;
use crate::client::{Client, ClientError};
use crate::component::{add_crate, Update};
use crate::component::add_crate::AddCrate;
use crate::widget::builder::WidgetBuilder;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct ViewCrates {
  add_crate: AddCrate,
  adding_crate: bool,
  crates_being_refreshed: BTreeSet<String>,
  all_crates_being_refreshed: bool,
  id_to_crate: HashMap<String, Crate>,
  client: Client,
}

#[derive(Default, Debug)]
pub enum Message {
  ToAddCrate(add_crate::Message),

  OpenAddCrateModal,
  CloseAddCrateModal,

  RefreshCrate(String),
  RefreshOutdated,
  RefreshAll,

  UpdateCrate(Result<Crate, ClientError>),
  UpdateCrates(Result<Vec<Crate>, ClientError>),

  RemoveCrate(String),

  #[default]
  Ignore,
}

impl ViewCrates {
  pub fn new(client: Client) -> (Self, Command<Message>) {
    let command = client.clone().search_crates(Search::followed()).perform(Message::UpdateCrates);
    let view_crates = Self {
      add_crate: Default::default(),
      adding_crate: false,
      crates_being_refreshed: Default::default(),
      all_crates_being_refreshed: false,
      id_to_crate: Default::default(),
      client,
    };
    (view_crates, command)
  }

  #[tracing::instrument(skip_all)]
  pub fn update(
    &mut self,
    message: Message,
    _data: &mut Data,
    _cache: &mut Cache
  ) -> Update<(), Command<Message>> {
    use Message::*;
    match message {
      ToAddCrate(message) => {
        let (action, command) = self.add_crate.update(message, &self.client).into_action_command();
        if let Some(krate) = action {
          self.id_to_crate.insert(krate.id.clone(), krate);
          self.add_crate.clear_search_term();
          self.adding_crate = false;
        }
        return command.map(ToAddCrate).into();
      }
      OpenAddCrateModal => {
        self.adding_crate = true;
        return self.add_crate.focus_search_term_input().into();
      }
      CloseAddCrateModal => {
        self.add_crate.clear_search_term();
        self.adding_crate = false;
      }

      RefreshCrate(crate_id) => {
        self.crates_being_refreshed.insert(crate_id.clone());
        return self.client.clone().refresh_crate(crate_id).perform(UpdateCrate).into();
      }
      RefreshOutdated => {
        self.all_crates_being_refreshed = true;
        return self.client.clone().refresh_outdated_crates().perform(UpdateCrates).into();
      }
      RefreshAll => {
        self.all_crates_being_refreshed = true;
        return self.client.clone().refresh_all_crates().perform(UpdateCrates).into();
      }

      UpdateCrate(Ok(krate)) => {
        let id = krate.id.clone();
        tracing::info!(id, "update crate");
        self.crates_being_refreshed.remove(&id);
        self.id_to_crate.insert(id, krate);
      }
      UpdateCrate(Err(cause)) => {
        tracing::error!(?cause, "failed to update crate");
        // TODO: remove crate id from `crates_being_refreshed`
      },
      UpdateCrates(Ok(crates)) => {
        for krate in crates {
          let id = krate.id.clone();
          tracing::info!(id, "update crate");
          self.crates_being_refreshed.remove(&id);
          self.id_to_crate.insert(id, krate);
        }
        self.all_crates_being_refreshed = false;
      }
      UpdateCrates(Err(cause)) => {
        tracing::error!(?cause, "failed to update crates");
        self.all_crates_being_refreshed = false;
      },

      RemoveCrate(id) => {
        self.crates_being_refreshed.remove(&id);
        self.id_to_crate.remove(&id);
      }

      Ignore => {}
    }
    Update::default()
  }

  #[tracing::instrument(skip_all)]
  pub fn view<'a>(&'a self, _data: &'a Data, _cache: &'a Cache) -> Element<'a, Message> {
    let cell_to_element = |row, col| -> Option<Element<'a, Message>> {
      let Some(krate) = self.id_to_crate.values().nth(row) else { return None; };
      match col {
        1 => return Some(WidgetBuilder::once().add_text(&krate.max_version)),
        2 => return Some(WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string())),
        3 => return Some(WidgetBuilder::once().add_text(format!("{}", krate.downloads))),
        _ => {}
      }
      let id = &krate.id;
      let element = match col {
        0 => WidgetBuilder::once().add_text(id),
        4 => WidgetBuilder::once().button("Refresh")
          .padding([1.0, 5.0])
          .on_press(|| Message::RefreshCrate(id.clone()))
          .disabled(self.all_crates_being_refreshed || self.crates_being_refreshed.contains(id))
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
      .body_row_count(self.id_to_crate.len())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(0.5, "")
      .push(0.5, "")
      .into_element();

    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .button("Add Crate").on_press(|| Message::OpenAddCrateModal).add()
      .button("Refresh Outdated").on_press(|| Message::RefreshOutdated).disabled(self.all_crates_being_refreshed || !self.crates_being_refreshed.is_empty()).add()
      .button("Refresh All").on_press(|| Message::RefreshAll).disabled(self.all_crates_being_refreshed || !self.crates_being_refreshed.is_empty()).add()
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
