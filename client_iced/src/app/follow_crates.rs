use std::time::Duration;

use iced::{Element, Task};
use tracing::instrument;

use att_client::crates::{Crates, CratesRequest, CratesResponse, CratesState};
use att_client::follow_crates::FollowCrates;
use att_client::http_client::AttHttpClient;
use att_client::query_sender::QuerySender;
use att_core::crates::{CratesQuery, CratesQueryConfig};
use att_core::iced_impls::as_full_table;
use iced_builder::{ElementExt, WidgetBuilder};

use crate::app::search_crates;
use crate::app::search_crates::SearchCratesComponent;
use crate::perform::{OptionPerformExt, PerformExt};
use crate::update::Update;
use crate::widget::modal::Modal;

pub struct FollowCratesComponent {
  crates: Crates,
  follow_crates: FollowCrates,
  search_crates: SearchCratesComponent,
  search_crates_modal_open: bool,
}

#[derive(Debug)]
pub enum Message {
  ToSearchCrates(search_crates::Message),
  OpenSearchCratesModal,
  CloseSearchCratesModal,
  SendRequest(CratesRequest),
  ProcessResponse(CratesResponse),
}

impl FollowCratesComponent {
  pub fn new(http_client: AttHttpClient, state: CratesState) -> Self {
    let query_sender = QuerySender::new(
      CratesQuery::from_followed(true),
      CratesQueryConfig {
        show_followed: false,
        ..CratesQueryConfig::default()
      },
      Duration::from_millis(300),
      true,
    );
    Self {
      crates: Crates::new(http_client.clone(), query_sender, state),
      follow_crates: FollowCrates,
      search_crates: SearchCratesComponent::new(http_client),
      search_crates_modal_open: false,
    }
  }

  pub fn state(&self) -> &CratesState {
    self.crates.state()
  }

  pub fn request_followed_crates(&mut self) -> Task<Message> {
    self.crates.send_initial_query().perform_into(Message::ProcessResponse)
  }

  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<(), Task<Message>> {
    use Message::*;
    match message {
      ToSearchCrates(message) => {
        let (action, command) = self.search_crates.update(message).into_action_task();
        let search_command = command.map(ToSearchCrates);
        if let Some(krate) = action {
          self.search_crates.reset();
          self.search_crates_modal_open = false;
          let follow_command = self.crates.send_follow(krate).perform_into(ProcessResponse);
          return Task::batch([search_command, follow_command]).into();
        }
        return search_command.into();
      }
      OpenSearchCratesModal => {
        self.search_crates_modal_open = true;
        return self.search_crates.focus_search_term_input().into();
      }
      CloseSearchCratesModal => {
        self.search_crates.reset();
        self.search_crates_modal_open = false;
      }
      SendRequest(request) => return self.crates.send(request).opt_perform(ProcessResponse).into(),
      ProcessResponse(response) => return self.crates.process(response).opt_perform(ProcessResponse).into(),
    }
    Update::default()
  }

  pub fn view(&self) -> Element<Message> {
    let custom_button = WidgetBuilder::once()
      .button("Add")
      .success_style()
      .on_press(|| Message::OpenSearchCratesModal)
      .add();
    let table = as_full_table(&self.crates, &self.follow_crates, Some("Followed Crates"), [custom_button], Message::SendRequest);

    if self.search_crates_modal_open {
      let overlay = self.search_crates
        .view()
        .map(Message::ToSearchCrates)
        .into_stack_builder()
        .container().padding(5).width(1200).height(900).add()
        .take();
      let modal = Modal::with_container(overlay, table)
        .on_close_modal(|| Message::CloseSearchCratesModal);
      modal.into()
    } else {
      table
    }
  }
}
