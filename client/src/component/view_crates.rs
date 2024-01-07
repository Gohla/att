use std::collections::{BTreeMap, BTreeSet};

use iced::{Command, Element};

use att_core::crates::{Crate, CrateSearch};

use crate::app::Cache;
use crate::client::{AttHttpClient, AttHttpClientError};
use crate::component::{follow_crate, Perform, Update};
use crate::component::follow_crate::FollowCrate;
use crate::widget::builder::WidgetBuilder;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct ViewCrates {
  follow_crate: FollowCrate,
  follow_crate_overlay_open: bool,

  crates_being_refreshed: BTreeSet<String>,
  all_crates_being_refreshed: bool,

  id_to_crate: BTreeMap<String, Crate>,
  client: AttHttpClient,
}

#[derive(Default, Debug)]
pub enum Message {
  ToAddCrate(follow_crate::Message),

  OpenAddCrateModal,
  CloseAddCrateModal,

  RefreshCrate(String),
  RefreshOutdated,
  RefreshAll,

  UpdateCrate(String, Result<Crate, AttHttpClientError>),
  UpdateCrates(Result<Vec<Crate>, AttHttpClientError>),
  SetCrates(Result<Vec<Crate>, AttHttpClientError>),

  RemoveCrate(String),

  #[default]
  Ignore,
}

impl ViewCrates {
  pub fn new(client: AttHttpClient, cache: &Cache) -> Self {
    Self {
      follow_crate: Default::default(),
      follow_crate_overlay_open: false,
      crates_being_refreshed: Default::default(),
      all_crates_being_refreshed: true,
      id_to_crate: cache.id_to_crate.clone(),
      client,
    }
  }

  pub fn request_followed_crates(&self) -> Command<Message> {
    self.client.clone().search_crates(CrateSearch::followed()).perform(Message::SetCrates)
  }

  #[tracing::instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<(), Command<Message>> {
    use Message::*;
    match message {
      ToAddCrate(message) => {
        let (action, command) = self.follow_crate.update(message, &self.client).into_action_command();
        if let Some(krate) = action {
          self.id_to_crate.insert(krate.id.clone(), krate);
          self.follow_crate.clear_search_term();
          self.follow_crate_overlay_open = false;
        }
        return command.map(ToAddCrate).into();
      }
      OpenAddCrateModal => {
        self.follow_crate_overlay_open = true;
        return self.follow_crate.focus_search_term_input().into();
      }
      CloseAddCrateModal => {
        self.follow_crate.clear_search_term();
        self.follow_crate_overlay_open = false;
      }

      RefreshCrate(crate_id) => {
        self.crates_being_refreshed.insert(crate_id.clone());
        return self.client.clone().refresh_crate(crate_id.clone())
          .perform(|r| UpdateCrate(crate_id, r)).into();
      }
      RefreshOutdated => {
        self.all_crates_being_refreshed = true;
        return self.client.clone().refresh_outdated_crates().perform(UpdateCrates).into();
      }
      RefreshAll => {
        self.all_crates_being_refreshed = true;
        return self.client.clone().refresh_all_crates().perform(SetCrates).into();
      }

      UpdateCrate(crate_id, result) => {
        self.crates_being_refreshed.remove(&crate_id);
        match result {
          Ok(krate) => {
            tracing::debug!(crate_id, "update crate");
            self.id_to_crate.insert(crate_id, krate);
          },
          Err(cause) => tracing::error!(?cause, "failed to update crate"),
        }
      }
      UpdateCrates(result) => {
        self.all_crates_being_refreshed = false;
        match result {
          Ok(crates) => {
            tracing::debug!(?crates, "update crates");
            for krate in crates {
              let crate_id = krate.id.clone();
              tracing::trace!(crate_id, "update crate");
              self.crates_being_refreshed.remove(&crate_id);
              self.id_to_crate.insert(crate_id, krate);
            }
          }
          Err(cause) => tracing::error!(?cause, "failed to update crates"),
        }
      }
      SetCrates(result) => {
        self.all_crates_being_refreshed = false;
        match result {
          Ok(crates) => {
            tracing::debug!(?crates, "set crates");
            self.id_to_crate = BTreeMap::from_iter(crates.into_iter().map(|c| (c.id.clone(), c)));
          }
          Err(cause) => tracing::error!(?cause, "failed to set crates"),
        }
      }

      RemoveCrate(id) => {
        self.crates_being_refreshed.remove(&id);
        self.id_to_crate.remove(&id);
      }

      Ignore => {}
    }
    Update::default()
  }

  #[tracing::instrument(skip(self))]
  pub fn view(&self) -> Element<Message> {
    let cell_to_element = |row, col| -> Option<Element<Message>> {
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

    let disable_refresh = self.all_crates_being_refreshed || !self.crates_being_refreshed.is_empty();
    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .button("Add").positive_style().on_press(|| Message::OpenAddCrateModal).add()
      .button("Refresh Outdated").on_press(|| Message::RefreshOutdated).disabled(disable_refresh).add()
      .button("Refresh All").on_press(|| Message::RefreshAll).disabled(disable_refresh).add()
      .add_space_fill_width()
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(table)
      .column().spacing(10.0).padding(10).fill().add()
      .take();

    if self.follow_crate_overlay_open {
      let overlay = self.follow_crate
        .view()
        .map(Message::ToAddCrate);
      let modal = Modal::with_container(overlay, content)
        .on_close_modal(|| Message::CloseAddCrateModal);
      modal.into()
    } else {
      content.into()
    }
  }

  pub fn cache(&mut self, cache: &mut Cache) {
    std::mem::swap(&mut self.id_to_crate, &mut cache.id_to_crate);
  }
}
