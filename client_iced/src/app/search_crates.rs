use std::time::Duration;

use iced::{Element, Task};
use iced::widget::text_input;
use tracing::instrument;

use att_client::crates::{Crates, CratesRequest, CratesResponse, CratesState};
use att_client::http_client::AttHttpClient;
use att_client::query_sender::QuerySender;
use att_client::search_crates::SearchCrates;
use att_core::crates::{CratesQuery, CratesQueryConfig, FullCrate};
use att_core::iced_impls::as_full_table;

use crate::perform::OptionPerformExt;
use crate::update::Update;

pub struct SearchCratesComponent {
  search_term_id: text_input::Id,
  crates: Crates,
  search_crates: SearchCrates,
}

#[derive(Debug)]
pub enum Message {
  SendRequest(CratesRequest),
  ProcessResponse(CratesResponse),
}

impl SearchCratesComponent {
  pub fn new(http_client: AttHttpClient) -> Self {
    let query_sender = QuerySender::new(
      CratesQuery::from_followed(false),
      CratesQueryConfig {
        show_followed: false,
        ..CratesQueryConfig::default()
      },
      Duration::from_millis(300),
      false,
    );
    Self {
      search_term_id: text_input::Id::unique(),
      crates: Crates::new(http_client, query_sender, CratesState::default()),
      search_crates: SearchCrates,
    }
  }

  pub fn focus_search_term_input<M: 'static>(&self) -> Task<M> {
    text_input::focus(self.search_term_id.clone())
  }

  pub fn reset(&mut self) {
    self.crates.reset();
  }
}

impl SearchCratesComponent {
  #[instrument(skip_all)]
  pub fn update(&mut self, message: Message) -> Update<Option<FullCrate>, Task<Message>> {
    use Message::*;
    match message {
      SendRequest(request) => match request {
        // HACK: Intercept follow and redirect to parent
        CratesRequest::Follow(full_crate) => Update::from_action(Some(full_crate)),
        _ => self.crates.send(request).opt_perform(ProcessResponse).into()
      },
      ProcessResponse(response) => self.crates.process(response).opt_perform(ProcessResponse).into(),
    }
  }

  pub fn view(&self) -> Element<Message> {
    as_full_table(&self.crates, &self.search_crates, None, [], Message::SendRequest)
  }
}
