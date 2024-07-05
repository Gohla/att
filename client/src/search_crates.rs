use att_core::action::{Action, ActionDef};
use att_core::crates::FullCrate;
use att_core::service::{DataActions, ServiceActions};

use crate::crates::{Crates, CratesRequest};

pub struct SearchCrates;

// Service actions

impl ServiceActions<Crates> for SearchCrates {
  fn action_definitions(&self, _crates: &Crates) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[];
    ACTION_DEFS
  }

  #[allow(refining_impl_trait_reachable, private_interfaces)]
  fn actions(&self, _crates: &Crates) -> impl IntoIterator<Item=ServiceAction> {
    []
  }
}

struct ServiceAction;

impl Action for ServiceAction {
  type Request = CratesRequest;

  #[inline]
  fn is_disabled(&self) -> bool { false }

  #[inline]
  fn request(&self) -> CratesRequest { CratesRequest::InitialQuery }
}

// Data actions

impl DataActions<Crates> for SearchCrates {
  fn data_action_definitions(&self, _crates: &Crates) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_table_row_text("Follow").with_success_style(),
    ];
    ACTION_DEFS
  }

  fn data_action<'d>(&self, _crates: &Crates, index: usize, full_crate: &'d FullCrate) -> Option<impl Action<Request=CratesRequest> + 'd> {
    let action = match index {
      0 => DataAction { kind: DataActionKind::Follow, full_crate: full_crate.clone() },
      _ => return None,
    };
    Some(action)
  }
}

struct DataAction {
  kind: DataActionKind,
  full_crate: FullCrate,
}

enum DataActionKind {
  Follow,
}

impl Action for DataAction {
  type Request = CratesRequest;

  #[inline]
  fn is_disabled(&self) -> bool { false }

  #[inline]
  fn request(&self) -> CratesRequest {
    match self.kind {
      DataActionKind::Follow => CratesRequest::Follow(self.full_crate.clone()),
    }
  }
}
