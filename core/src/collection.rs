use iced::alignment::{Horizontal, Vertical};
use iced::Font;

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
  type Request;
  fn is_disabled(&self) -> bool;
  fn request(&self) -> Self::Request;
}

pub struct ActionWithDef<'a, A> {
  pub definition: &'a ActionDef,
  pub action: A,
}

impl<'a, A: Action + 'a> From<(&'a ActionDef, A)> for ActionWithDef<'a, A> {
  #[inline]
  fn from((definition, action): (&'a ActionDef, A)) -> Self {
    ActionWithDef { definition, action }
  }
}

#[cfg(feature = "iced")]
// TODO: generic theme and renderer? causes a lot of type errors that I couldn't solve.
impl<'a, A> From<ActionWithDef<'a, A>> for iced::Element<'a, A::Request> where
  A: Action + 'a,
{
  fn from(ActionWithDef { definition, action }: ActionWithDef<A>) -> Self {
    let mut content = iced_builder::WidgetBuilder::once().text(definition.text);
    if let Some(font_name) = definition.font_name {
      content = content.font(Font::with_name(font_name));
    }
    if definition.icon {
      content = content
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .line_height(1.0)
    }
    let content: iced::Element<'a, ()> = content.add();

    let mut button = iced_builder::WidgetBuilder::once()
      .button(content)
      .disabled(action.is_disabled())
      .on_press(move || action.request())
      ;
    if definition.icon {
      button = button.padding(3.0);
    }
    button = match definition.style {
      ActionStyle::Primary => button.primary_style(),
      ActionStyle::Secondary => button.secondary_style(),
      ActionStyle::Success => button.success_style(),
      ActionStyle::Danger => button.danger_style(),
    };
    button.add()
  }
}

pub trait Collection {
  fn action_definitions(&self) -> &[ActionDef];

  fn actions(&self) -> impl IntoIterator<Item=impl Action<Request=Self::Request>>;

  #[inline]
  fn actions_with_definitions(&self) -> impl Iterator<Item=ActionWithDef<impl Action<Request=Self::Request>>> {
    self.action_definitions().iter().zip(self.actions()).map(Into::into)
  }


  type Item;

  fn item_action_definitions(&self) -> &[ActionDef];

  fn item_action<'i>(&self, index: usize, item: &'i Self::Item) -> Option<impl Action<Request=Self::Request> + 'i>;

  #[inline]
  fn item_action_with_definition<'i>(&self, index: usize, item: &'i Self::Item) -> Option<ActionWithDef<impl Action<Request=Self::Request> + 'i>> {
    match (self.item_action_definitions().get(index), self.item_action(index, item)) {
      (Some(definition), Some(action)) => Some(ActionWithDef { definition, action }),
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
