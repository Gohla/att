use att_core::action::{Action, ActionDef};
use att_core::crates::FullCrate;
use att_core::service::{DataActions, ServiceActions};

use crate::crates::{Crates, CratesRequest};

pub struct FollowCrates;

// Service actions

impl ServiceActions<Crates> for FollowCrates {
  fn action_definitions(&self, _crates: &Crates) -> &[ActionDef] {
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_text("Refresh Followed"),
    ];
    ACTION_DEFS
  }

  fn actions(&self, crates: &Crates) -> impl IntoIterator<Item=impl Action<Request=CratesRequest>> {
    let disabled = crates.are_all_crates_being_modified();
    [
      ServiceAction { kind: ServiceActionKind::RefreshFollowed, disabled },
    ]
  }
}

struct ServiceAction {
  kind: ServiceActionKind,
  disabled: bool,
}

enum ServiceActionKind {
  RefreshFollowed,
}

impl Action for ServiceAction {
  type Request = CratesRequest;

  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> CratesRequest {
    match self.kind {
      ServiceActionKind::RefreshFollowed => CratesRequest::RefreshFollowed,
    }
  }
}

// Data actions

impl DataActions<Crates> for FollowCrates {
  fn data_action_definitions(&self, _crates: &Crates) -> &[ActionDef] {
    const ICON_FONT: &'static str = "bootstrap-icons";
    const ACTION_DEFS: &'static [ActionDef] = &[
      ActionDef::from_table_row_icon("\u{F116}", ICON_FONT),
      ActionDef::from_table_row_icon("\u{F5DE}", ICON_FONT).with_danger_style(),
    ];
    ACTION_DEFS
  }

  fn data_action<'d>(&self, crates: &Crates, index: usize, full_crate: &'d FullCrate) -> Option<impl Action<Request=CratesRequest> + 'd> {
    let crate_id = full_crate.krate.id;
    let disabled = crates.is_crate_being_modified(crate_id);
    let action = match index {
      0 => DataAction { kind: DataActionKind::Refresh, disabled, crate_id },
      1 => DataAction { kind: DataActionKind::Unfollow, disabled, crate_id },
      _ => return None,
    };
    Some(action)
  }
}

struct DataAction {
  kind: DataActionKind,
  disabled: bool,
  crate_id: i32,
}

enum DataActionKind {
  Refresh,
  Unfollow,
}

impl Action for DataAction {
  type Request = CratesRequest;

  #[inline]
  fn is_disabled(&self) -> bool { self.disabled }

  #[inline]
  fn request(&self) -> CratesRequest {
    match self.kind {
      DataActionKind::Refresh => CratesRequest::Refresh(self.crate_id),
      DataActionKind::Unfollow => CratesRequest::Unfollow(self.crate_id),
    }
  }
}
