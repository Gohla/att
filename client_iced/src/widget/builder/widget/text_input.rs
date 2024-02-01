use iced::advanced::text::Renderer as TextRenderer;
use iced::Element;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use iced::widget::TextInput;

use super::super::state::State;

pub trait TextInputActions<'a, M> {
  type Change;

  fn on_input(self, on_input: impl Fn(String) -> M + 'a) -> Self::Change;
  fn on_paste(self, on_paste: impl Fn(String) -> M + 'a) -> Self::Change;
  fn on_submit(self, on_submit: impl Fn() -> M + 'a) -> Self::Change;
}

type TI<'a, M, S> = TextInput<'a, M, <S as State>::Theme, <S as State>::Renderer>;

pub trait CreateTextInput<'a, S> where
  S: State,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet
{
  type Message: Clone;

  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TI<'a, Self::Message, S>) -> TI<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer>;
}

/// Passthrough which does not modify the message type.
pub struct TextInputPassthrough;
impl<'a, M> TextInputActions<'a, M> for TextInputPassthrough {
  type Change = TextInputFunctions<'a, M>;

  #[inline]
  fn on_input(self, on_input: impl Fn(String) -> M + 'a) -> Self::Change {
    TextInputFunctions { on_input: Some(Box::new(on_input)), ..Default::default() }
  }
  #[inline]
  fn on_paste(self, on_paste: impl Fn(String) -> M + 'a) -> Self::Change {
    TextInputFunctions { on_paste: Some(Box::new(on_paste)), ..Default::default() }
  }
  #[inline]
  fn on_submit(self, on_submit: impl Fn() -> M + 'a) -> Self::Change {
    TextInputFunctions { on_submit: Some(Box::new(on_submit)), ..Default::default() }
  }
}
impl<'a, S> CreateTextInput<'a, S> for TextInputPassthrough where
  S: State + 'a,
  S::Message: Clone,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
{
  type Message = S::Message;

  #[inline]
  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TI<'a, Self::Message, S>) -> TI<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer> {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    Element::new(text_input)
  }
}

/// Modify message type to [`TextInputAction`] which is [`Clone`], without our callbacks needing to implement clone.
pub struct TextInputFunctions<'a, M> {
  on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
  on_paste: Option<Box<dyn Fn(String) -> M + 'a>>,
  on_submit: Option<Box<dyn Fn() -> M + 'a>>,
}
impl<'a, M> Default for TextInputFunctions<'a, M> {
  fn default() -> Self { Self { on_input: None, on_paste: None, on_submit: None } }
}
impl<'a, M> TextInputActions<'a, M> for TextInputFunctions<'a, M> {
  type Change = Self;

  #[inline]
  fn on_input(mut self, on_input: impl Fn(String) -> M + 'a) -> Self::Change {
    self.on_input = Some(Box::new(on_input));
    self
  }
  #[inline]
  fn on_paste(mut self, on_paste: impl Fn(String) -> M + 'a) -> Self::Change {
    self.on_paste = Some(Box::new(on_paste));
    self
  }
  #[inline]
  fn on_submit(mut self, on_submit: impl Fn() -> M + 'a) -> Self::Change {
    self.on_submit = Some(Box::new(on_submit));
    self
  }
}
impl<'a, S> CreateTextInput<'a, S> for TextInputFunctions<'a, S::Message> where
  S: State + 'a,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
{
  type Message = TextInputAction;

  #[inline]
  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TI<'a, Self::Message, S>) -> TI<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer> {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    if self.on_input.is_some() {
      text_input = text_input.on_input(TextInputAction::Input);
    }
    if self.on_paste.is_some() {
      text_input = text_input.on_paste(TextInputAction::Paste);
    }
    if self.on_submit.is_some() {
      text_input = text_input.on_submit(TextInputAction::Submit);
    }
    Element::new(text_input)
      .map(move |m| match m {
        TextInputAction::Input(input) => (self.on_input.as_ref().unwrap())(input),
        TextInputAction::Paste(input) => (self.on_paste.as_ref().unwrap())(input),
        TextInputAction::Submit => (self.on_submit.as_ref().unwrap())(),
      })
  }
}

#[derive(Clone)]
pub enum TextInputAction {
  Input(String),
  Paste(String),
  Submit,
}
