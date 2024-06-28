use std::iter;
use iced::{Element, Task};
use iced::widget::Row;
use tracing::instrument;

use att_client::follow_crates::{FollowCrateRequest, FollowCrates, FollowCratesData, FollowCratesResponse};
use att_client::http_client::AttHttpClient;
use att_core::collection::{Action, ActionStyle, Collection};
use att_core::crates::Crate;
use att_core::table::AsTableRow;
use iced_builder::WidgetBuilder;

use crate::app::search_crates;
use crate::app::search_crates::SearchCratesComponent;
use crate::perform::PerformExt;
use crate::update::Update;
use crate::widget::constrained_row::Constraint;
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
      if let Some(text) = krate.cell(col as u8) {
        return Some(WidgetBuilder::once().add_text(text))
      }

      let action_index = col - Crate::COLUMNS.len();
      let element = if let Some((action_def, action)) = self.follow_crates.item_action_with_definition(action_index, krate) {
        let button = WidgetBuilder::once()
          .button(icon_text(action_def.text))
          .padding(4.0)
          .disabled(action.is_disabled())
          .on_press(move || Message::SendRequest(action.request()))
          ;
        let button = match action_def.style {
          ActionStyle::Primary => button.primary_style(),
          ActionStyle::Secondary => button.secondary_style(),
          ActionStyle::Success => button.success_style(),
          ActionStyle::Danger => button.danger_style(),
        };
        button.add()
      } else {
        return None
      };
      Some(element)
    };
    let mut table = Table::with_capacity(5, cell_to_element)
      .spacing(1.0)
      .body_row_height(24.0)
      .body_row_count(data.num_followed_crates());
    for column in Crate::COLUMNS {
      table = table.push(Constraint::new(column.width_fill_portion, column.horizontal_alignment.into(), column.vertical_alignment.into()), column.header)
    }
    for _ in self.follow_crates.item_action_definitions() {
      table = table.push(0.2, "");
    }
    let table = table.into_element();

    let custom_button = WidgetBuilder::once().button("Add").success_style().on_press(|| Message::OpenSearchCratesModal).add();
    let action_buttons = self.follow_crates.actions_with_definitions().map(|(action_def, action)| WidgetBuilder::once()
      .button(action_def.text)
      .disabled(action.is_disabled())
      .on_press(move || Message::SendRequest(action.request()))
      .add()
    );
    let buttons: Vec<_> = iter::once(custom_button).chain(action_buttons).collect();

    let content = WidgetBuilder::stack()
      .text("Followed Crates").size(20.0).add()
      .add_element(Row::from_vec(buttons).spacing(5.0))
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
