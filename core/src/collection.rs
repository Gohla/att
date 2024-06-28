use crate::util::maybe_send::MaybeSendFuture;

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionStyle {
  #[default]
  Primary,
  Secondary,
  Success,
  Danger
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionDef {
  pub text: &'static str,
  pub style: ActionStyle,
}

impl ActionDef {
  #[inline]
  pub const fn new(text: &'static str, style: ActionStyle) -> Self { Self { text, style } }
  #[inline]
  pub const fn from_text(text: &'static str) -> Self { Self::new(text, ActionStyle::Primary) }

  #[inline]
  pub const fn with_style(mut self, style: ActionStyle) -> Self {
    self.style = style;
    self
  }
  #[inline]
  pub const fn with_primary_style(self) -> Self { self.with_style(ActionStyle::Primary) }
  #[inline]
  pub const fn with_secondary_style(self) -> Self { self.with_style(ActionStyle::Secondary) }
  #[inline]
  pub const fn with_success_style(self) -> Self { self.with_style(ActionStyle::Success) }
  #[inline]
  pub const fn with_danger_style(self) -> Self { self.with_style(ActionStyle::Danger) }
}

pub trait Action<Request> {
  fn is_disabled(&self) -> bool;
  fn request(&self) -> Request;
}

pub trait Collection {
  fn action_definitions(&self) -> &[ActionDef];

  fn actions(&self) -> impl IntoIterator<Item=impl Action<Self::Request>>;

  #[inline]
  fn actions_with_definitions(&self) -> impl Iterator<Item=(&ActionDef, impl Action<Self::Request>)> {
    self.action_definitions().iter().zip(self.actions())
  }


  type Item;

  fn item_action_definitions(&self) -> &[ActionDef];

  fn item_action<'i>(&self, index: usize, item: &'i Self::Item) -> Option<impl Action<Self::Request> + 'i>;

  #[inline]
  fn item_action_with_definition<'i>(&self, index: usize, item: &'i Self::Item) -> Option<(&ActionDef, impl Action<Self::Request> + 'i)> {
    match (self.item_action_definitions().get(index), self.item_action(index, item)) {
      (Some(action_def), Some(action)) => Some((action_def, action)),
      _ => None
    }
  }


  type Request;
  type Response;

  /// Send `request`, creating a future that produces a response when completed. The response must be
  /// [processed](Self::process).
  fn send(&mut self, request: Self::Request) -> impl MaybeSendFuture<'static, Output=Self::Response>;

  type Data;

  /// Process `response` that a future, created by [send](Self::send), returned on completion.
  fn process(&mut self, data: &mut Self::Data, response: Self::Response);
}
