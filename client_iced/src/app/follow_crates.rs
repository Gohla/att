use iced::{Element, Task};
use tracing::instrument;

use att_client::follow_crates::{FollowCrateRequest, FollowCrates, FollowCratesData, FollowCratesResponse};
use att_client::http_client::AttHttpClient;
use iced_builder::WidgetBuilder;

use crate::app::search_crates;
use crate::app::search_crates::SearchCratesComponent;
use crate::perform::PerformExt;
use crate::update::Update;
use crate::widget::icon::icon_text;
use crate::widget::modal::Modal;
use crate::widget::table::Table;
use crate::widget::WidgetExt;

pub struct FollowCratesComponent {
  follow_crates: FollowCrates,
  search_crates: SearchCratesComponent,
  search_crates_modal_open: bool,
}

#[derive(Debug)]
pub enum Message {
  ToSearchCrates(search_crates::Message),
  OpenSearchCratesModal,
  CloseSearchCratesModal,
  SendRequest(FollowCrateRequest),
  ProcessResponse(FollowCratesResponse),
}

impl FollowCratesComponent {
  pub fn new(http_client: AttHttpClient) -> Self {
    Self {
      follow_crates: FollowCrates::new(http_client.clone()),
      search_crates: SearchCratesComponent::new(http_client, "Follow"),
      search_crates_modal_open: false,
    }
  }

  pub fn request_followed_crates(&mut self) -> Task<Message> {
    self.follow_crates.get_followed().perform_into(Message::ProcessResponse)
  }

  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message, data: &mut FollowCratesData) -> Update<(), Task<Message>> {
    use Message::*;
    match message {
      ToSearchCrates(message) => {
        let (action, command) = self.search_crates.update(message).into_action_task();
        let search_command = command.map(ToSearchCrates);
        if let Some(crate_id) = action {
          self.search_crates.clear();
          self.search_crates_modal_open = false;
          let follow_command = self.follow_crates.follow(crate_id).perform_into(ProcessResponse);
          return Task::batch([search_command, follow_command]).into();
        }
        return search_command.into();
      }
      OpenSearchCratesModal => {
        self.search_crates_modal_open = true;
        return self.search_crates.focus_search_term_input().into();
      }
      CloseSearchCratesModal => {
        self.search_crates.clear();
        self.search_crates_modal_open = false;
      }
      SendRequest(request) => return self.follow_crates.send(request).perform(ProcessResponse).into(),
      ProcessResponse(response) => self.follow_crates.process(response, data),
    }
    Update::default()
  }

  pub fn view<'a>(&'a self, data: &'a FollowCratesData) -> Element<'a, Message> {
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
          .on_press(|| Message::SendRequest(FollowCrateRequest::Refresh(crate_id.clone())))
          .disabled(self.follow_crates.is_crate_being_modified(crate_id))
          .add(),
        5 => WidgetBuilder::once()
          .button(icon_text("\u{F5DE}"))
          .danger_style()
          .padding(4.0)
          .on_press(|| Message::SendRequest(FollowCrateRequest::Unfollow(crate_id.clone())))
          .disabled(self.follow_crates.is_crate_being_modified(crate_id))
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

    let disable_refresh = self.follow_crates.is_any_crate_being_modified();
    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .button("Add").success_style().on_press(|| Message::OpenSearchCratesModal).add()
      .button("Refresh Outdated").on_press(|| Message::SendRequest(FollowCrateRequest::RefreshOutdated)).disabled(disable_refresh).add()
      .button("Refresh All").on_press(|| Message::SendRequest(FollowCrateRequest::RefreshAll)).disabled(disable_refresh).add()
      .add_space_fill_width()
      .row().spacing(10.0).align_center().fill_width().add()
      .add_horizontal_rule(1.0)
      .add_element(table)
      .column().spacing(10.0).padding(10).fill().add()
      .take();

    if self.search_crates_modal_open {
      let overlay = self.search_crates
        .view()
        .map(Message::ToSearchCrates);
      let modal = Modal::with_container(overlay, content)
        .on_close_modal(|| Message::CloseSearchCratesModal);
      modal.into()
    } else {
      content
    }
  }
}
