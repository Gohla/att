use iced::{Command, Element};
use tracing::instrument;

use att_client::AttClient;
use att_client::crates::{CrateAction, CrateData, CrateOperation, CrateRequest, CrateViewData};

use crate::component::{Perform, PerformInto, search_crate, Update};
use crate::component::search_crate::SearchCrates;
use crate::widget::builder::WidgetBuilder;
use crate::widget::icon::icon_text;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct ViewFollowedCrates {
  request: CrateRequest,
  search_crate: SearchCrates,
  view_data: CrateViewData,
  search_overlay_open: bool,
}

#[derive(Debug)]
pub enum Message {
  ToSearchCrate(search_crate::Message),
  OpenSearchCrateModal,
  CloseSearchCrateModal,

  PerformAction(CrateAction),
  ApplyOperation(CrateOperation),
}

impl ViewFollowedCrates {
  pub fn new(client: AttClient) -> Self {
    Self {
      request: client.crates(),
      search_crate: SearchCrates::new(client.http_client().clone(), "Follow"),
      view_data: CrateViewData::default(),
      search_overlay_open: false,
    }
  }

  pub fn request_followed_crates(&mut self) -> Command<Message> {
    self.request.clone().get_followed(&mut self.view_data).perform_into(Message::ApplyOperation)
  }

  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message, data: &mut CrateData) -> Update<(), Command<Message>> {
    use Message::*;
    match message {
      ToSearchCrate(message) => {
        let (action, command) = self.search_crate.update(message).into_action_command();
        let search_command = command.map(ToSearchCrate);
        if let Some(crate_id) = action {
          self.search_crate.clear();
          self.search_overlay_open = false;
          let follow_command = self.request.clone().follow(&mut self.view_data, crate_id).perform_into(ApplyOperation);
          return Command::batch([search_command, follow_command]).into();
        }
        return search_command.into();
      }
      OpenSearchCrateModal => {
        self.search_overlay_open = true;
        return self.search_crate.focus_search_term_input().into();
      }
      CloseSearchCrateModal => {
        self.search_crate.clear();
        self.search_overlay_open = false;
      }

      PerformAction(action) => return action.perform(self.request.clone(), &mut self.view_data).perform(ApplyOperation).into(),
      ApplyOperation(operation) => operation.apply(&mut self.view_data, data),
    }
    Update::default()
  }

  pub fn view<'a>(&'a self, data: &'a CrateData) -> Element<Message> {
    let cell_to_element = |row, col| -> Option<Element<Message>> {
      let Some(krate) = data.followed_crates().nth(row) else { return None; };
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
          .on_press(|| Message::PerformAction(CrateAction::Refresh(crate_id.clone())))
          .disabled(self.view_data.is_crate_being_modified(crate_id))
          .add(),
        5 => WidgetBuilder::once()
          .button(icon_text("\u{F5DE}"))
          .destructive_style()
          .padding(4.0)
          .on_press(|| Message::PerformAction(CrateAction::Unfollow(crate_id.clone())))
          .disabled(self.view_data.is_crate_being_modified(crate_id))
          .add(),
        _ => return None,
      };
      Some(element)
    };
    let table = Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(data.num_followed_crates())
      .push(2, "Name")
      .push(1, "Latest Version")
      .push(1, "Updated at")
      .push(1, "Downloads")
      .push(0.2, "")
      .push(0.2, "")
      .into_element();

    let disable_refresh = self.view_data.is_any_crate_being_modified();
    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .button("Add").positive_style().on_press(|| Message::OpenSearchCrateModal).add()
      .button("Refresh Outdated").on_press(|| Message::PerformAction(CrateAction::RefreshOutdated)).disabled(disable_refresh).add()
      .button("Refresh All").on_press(|| Message::PerformAction(CrateAction::RefreshAll)).disabled(disable_refresh).add()
      .add_space_fill_width()
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(table)
      .column().spacing(10.0).padding(10).fill().add()
      .take();

    if self.search_overlay_open {
      let overlay = self.search_crate
        .view()
        .map(Message::ToSearchCrate);
      let modal = Modal::with_container(overlay, content)
        .on_close_modal(|| Message::CloseSearchCrateModal);
      modal.into()
    } else {
      content.into()
    }
  }
}
