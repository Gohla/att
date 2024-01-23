use iced::{Command, Element};
use tracing::instrument;

use att_client::{AttClient, Data, RemoveCrate, UpdateCrate, UpdateCrates, ViewData};

use crate::component::{follow_crate, Perform, Update};
use crate::component::follow_crate::FollowCrate;
use crate::widget::builder::WidgetBuilder;
use crate::widget::icon::icon_text;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct ViewCrates {
  follow_crate: FollowCrate,
  follow_crate_overlay_open: bool,

  client: AttClient,
}

#[derive(Debug)]
pub enum Message {
  ToFollowCrate(follow_crate::Message),

  OpenFollowCrateModal,
  CloseFollowCrateModal,

  RefreshCrate(String),
  RefreshOutdated,
  RefreshAll,
  UnfollowCrate(String),

  SetCrates(UpdateCrates<true>),
  UpdateCrates(UpdateCrates<false>),
  UpdateCrate(UpdateCrate),
  RemoveCrate(RemoveCrate),
}

impl ViewCrates {
  pub fn new(client: AttClient) -> Self {
    Self {
      follow_crate: Default::default(),
      follow_crate_overlay_open: false,
      client,
    }
  }

  pub fn request_followed_crates(&self, view_data: &mut ViewData) -> Command<Message> {
    self.client.clone().get_followed_crates(view_data).perform(Message::SetCrates)
  }

  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message, data: &mut Data, view_data: &mut ViewData) -> Update<(), Command<Message>> {
    use Message::*;
    match message {
      ToFollowCrate(message) => {
        let (action, command) = self.follow_crate.update(message, self.client.http_client()).into_action_command();
        if let Some(krate) = action {
          data.id_to_crate.insert(krate.id.clone(), krate);
          self.follow_crate.clear_search_term();
          self.follow_crate_overlay_open = false;
        }
        return command.map(ToFollowCrate).into();
      }
      OpenFollowCrateModal => {
        self.follow_crate_overlay_open = true;
        return self.follow_crate.focus_search_term_input().into();
      }
      CloseFollowCrateModal => {
        self.follow_crate.clear_search_term();
        self.follow_crate_overlay_open = false;
      }

      RefreshCrate(crate_id) => {
        return self.client.clone().refresh_crate(view_data, crate_id.clone()).perform(UpdateCrate).into();
      }
      RefreshOutdated => {
        return self.client.clone().refresh_outdated_crates(view_data).perform(UpdateCrates).into();
      }
      RefreshAll => {
        return self.client.clone().refresh_all_crates(view_data).perform(SetCrates).into();
      }
      UnfollowCrate(crate_id) => {
        return self.client.clone().unfollow_crate(view_data, crate_id.clone()).perform(RemoveCrate).into();
      }

      UpdateCrate(operation) => {
        let _ = operation.apply(data, view_data);
      }
      UpdateCrates(operation) => {
        let _ = operation.apply(data, view_data);
      }
      SetCrates(operation) => {
        let _ = operation.apply(data, view_data);
      }
      RemoveCrate(operation) => {
        let _ = operation.apply(data, view_data);
      }
    }
    Update::default()
  }

  pub fn view<'a>(&'a self, data: &'a Data, view_data: &'a ViewData) -> Element<Message> {
    let cell_to_element = |row, col| -> Option<Element<Message>> {
      let Some(krate) = data.id_to_crate.values().nth(row) else { return None; };
      match col {
        1 => return Some(WidgetBuilder::once().add_text(&krate.max_version)),
        2 => return Some(WidgetBuilder::once().add_text(krate.updated_at.format("%Y-%m-%d").to_string())),
        3 => return Some(WidgetBuilder::once().add_text(format!("{}", krate.downloads))),
        _ => {}
      }
      let crate_id = &krate.id;
      let element = match col {
        0 => WidgetBuilder::once().add_text(crate_id),
        4 => WidgetBuilder::once()
          .button(icon_text("\u{F116}"))
          .padding(4.0)
          .on_press(|| Message::RefreshCrate(crate_id.clone()))
          .disabled(view_data.is_crate_being_modified(crate_id))
          .add(),
        5 => WidgetBuilder::once()
          .button(icon_text("\u{F5DE}"))
          .destructive_style()
          .padding(4.0)
          .on_press(|| Message::UnfollowCrate(crate_id.clone()))
          .disabled(view_data.is_crate_being_modified(crate_id))
          .add(),
        _ => return None,
      };
      Some(element)
    };
    let table = Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(data.id_to_crate.len())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(0.2, "")
      .push(0.2, "")
      .into_element();

    let disable_refresh = view_data.is_any_crate_being_modified();
    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .button("Add").positive_style().on_press(|| Message::OpenFollowCrateModal).add()
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
        .map(Message::ToFollowCrate);
      let modal = Modal::with_container(overlay, content)
        .on_close_modal(|| Message::CloseFollowCrateModal);
      modal.into()
    } else {
      content.into()
    }
  }
}
