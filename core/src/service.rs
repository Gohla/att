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
  pub font_name: Option<&'static str>, // TODO: abstract over icon/font name
  pub icon: bool,
  pub style: ActionStyle,
}

impl ActionDef {
  #[inline]
  pub const fn new(text: &'static str, font_name: Option<&'static str>, icon: bool, style: ActionStyle) -> Self {
    Self { text, font_name, icon, style }
  }
  #[inline]
  pub const fn from_text(text: &'static str) -> Self {
    Self::new(text, None, false, ActionStyle::Primary)
  }
  #[inline]
  pub const fn from_icon_font(icon: &'static str, font_name: &'static str) -> Self {
    Self::new(icon, Some(font_name), true, ActionStyle::Primary)
  }

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

pub trait Action {
  fn is_disabled(&self) -> bool;

  type Request;
  fn request(&self) -> Self::Request;
}

pub struct ActionWithDef<'a, A> {
  pub definition: &'a ActionDef,
  pub action: A,
}
impl<'a, A: Action> From<(&'a ActionDef, A)> for ActionWithDef<'a, A> {
  #[inline]
  fn from((definition, action): (&'a ActionDef, A)) -> Self {
    ActionWithDef { definition, action }
  }
}

/// Service that sends requests and processes responses for a collection of data.
///
/// A service has [service-wide actions](Self::actions) and [data-specific actions](Self::data_actions) that produce a
/// [`Request`](Self::Request). Requests are [sent](Self::send), creating a future that returns a
/// [`Response`](Self::Response) on completion. Responses must be [processed](Self::process).
pub trait Service {
  fn action_definitions(&self) -> &[ActionDef];

  fn actions(&self) -> impl IntoIterator<Item=impl Action<Request=Self::Request>>;

  #[inline]
  fn actions_with_definitions(&self) -> impl Iterator<Item=ActionWithDef<impl Action<Request=Self::Request>>> {
    self.action_definitions().iter().zip(self.actions()).map(Into::into)
  }


  type Data;

  fn data_len(&self) -> usize;

  fn get_data(&self, index: usize) -> Option<&Self::Data>;

  fn iter_data(&self) -> impl Iterator<Item=&Self::Data>;



  fn data_action_definitions(&self) -> &[ActionDef];

  fn data_action<'d>(&self, index: usize, data: &'d Self::Data) -> Option<impl Action<Request=Self::Request> + 'd>;

  #[inline]
  fn data_action_with_definition<'d>(&self, index: usize, data: &'d Self::Data) -> Option<ActionWithDef<impl Action<Request=Self::Request> + 'd>> {
    match (self.data_action_definitions().get(index), self.data_action(index, data)) {
      (Some(definition), Some(action)) => Some(ActionWithDef { definition, action }),
      _ => None
    }
  }


  type Request;
  type Response;

  /// Send `request`, creating a future that produces a response when completed. The response must be
  /// [processed](Self::process).
  fn send(&mut self, request: Self::Request) -> impl MaybeSendFuture<'static, Output=Self::Response> + 'static;

  /// Process `response` (that a future, created by [send](Self::send), returned on completion) into `self`.
  fn process(&mut self, response: Self::Response);
}
