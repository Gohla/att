use iced::{Element, Task};
use tracing::instrument;

use att_client::follow_crates::{FollowCrateRequest, FollowCrates, FollowCratesResponse, FollowCratesState};
use att_client::http_client::AttHttpClient;
use att_core::iced_impls::{as_table, QueryMessage, update_query};
use att_core::service::Service;
use iced_builder::WidgetBuilder;

use crate::app::search_crates;
use crate::app::search_crates::SearchCratesComponent;
use crate::perform::PerformExt;
use crate::update::Update;
use crate::widget::modal::Modal;

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
  Query(QueryMessage),
}

impl FollowCratesComponent {
  pub fn new(http_client: AttHttpClient, state: FollowCratesState) -> Self {
    Self {
      follow_crates: FollowCrates::new(http_client.clone(), state),
      search_crates: SearchCratesComponent::new(http_client, "Follow"),
      search_crates_modal_open: false,
    }
  }

  pub fn state(&self) -> &FollowCratesState {
    self.follow_crates.state()
  }

  pub fn request_followed_crates(&mut self) -> Task<Message> {
    self.follow_crates.get_followed().perform_into(Message::ProcessResponse)
  }

  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<(), Task<Message>> {
    use Message::*;
    match message {
      ToSearchCrates(message) => {
        let (action, command) = self.search_crates.update(message).into_action_task();
        let search_command = command.map(ToSearchCrates);
        if let Some(krate) = action {
          self.search_crates.clear();
          self.search_crates_modal_open = false;
          let follow_command = self.follow_crates.follow(krate).perform_into(ProcessResponse);
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
      ProcessResponse(response) => self.follow_crates.process(response),
      Query(message) => {
        update_query(self.follow_crates.query_mut(), message);
      }
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let custom_button = WidgetBuilder::once()
      .button("Add")
      .success_style()
      .on_press(|| Message::OpenSearchCratesModal)
      .add();
    let table = as_table(&self.follow_crates, "Followed Crates", Message::SendRequest, Message::Query, [custom_button]);

    if self.search_crates_modal_open {
      let overlay = self.search_crates
        .view()
        .map(Message::ToSearchCrates);
      let modal = Modal::with_container(overlay, table)
        .on_close_modal(|| Message::CloseSearchCratesModal);
      modal.into()
    } else {
      table
    }
  }
}
