use iced::advanced::text::Renderer as TextRenderer;
use iced::Element;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use iced::widget::TextInput;

use crate::widget::builder::state::Elem;
use crate::widget::builder::util::{TNone, TOption, TOptionFn, TSome};

use super::super::state::State;

pub trait TextInputActions {
  type ChangeOnInput<F>;
  fn on_input<F>(self, on_input: F) -> Self::ChangeOnInput<F>;
  type ChangeOnPaste<F>;
  fn on_paste<F>(self, on_paste: F) -> Self::ChangeOnPaste<F>;
  type ChangeOnSubmit<F>;
  fn on_submit<F>(self, on_submit: F) -> Self::ChangeOnSubmit<F>;
}

type TextIn<'a, S, M> = TextInput<'a, M, <S as State>::Theme, <S as State>::Renderer>;

pub trait CreateTextInput<'a, S> where
  S: State,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
{
  type Message: Clone;
  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TextIn<'a, S, Self::Message>) -> TextIn<'a, S, Self::Message>,
  ) -> Elem<'a, S>;
}

/// Passthrough which does not modify the message type, thus the message type must implement [`Clone`].
pub struct TextInputPassthrough;

impl TextInputActions for TextInputPassthrough {
  type ChangeOnInput<F> = <TextInputFunctions as TextInputActions>::ChangeOnInput<F>;
  #[inline]
  fn on_input<F>(self, on_input: F) -> Self::ChangeOnInput<F> { TextInputFunctions::default().on_input(on_input) }
  type ChangeOnPaste<F> = <TextInputFunctions as TextInputActions>::ChangeOnPaste<F>;
  #[inline]
  fn on_paste<F>(self, on_paste: F) -> Self::ChangeOnPaste<F> { TextInputFunctions::default().on_paste(on_paste) }
  type ChangeOnSubmit<F> = <TextInputFunctions as TextInputActions>::ChangeOnSubmit<F>;
  #[inline]
  fn on_submit<F>(self, on_submit: F) -> Self::ChangeOnSubmit<F> { TextInputFunctions::default().on_submit(on_submit) }
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
    modify: impl FnOnce(TextIn<'a, S, Self::Message>) -> TextIn<'a, S, Self::Message>,
  ) -> Elem<'a, S> {
    Element::new(modify(TextInput::new(placeholder, value)))
  }
}

/// Modify message type to [`TextInputAction`] which is [`Clone`], without our callbacks needing to implement clone.
pub struct TextInputFunctions<FI = TNone, FP = TNone, FS = TNone> {
  on_input: FI,
  on_paste: FP,
  on_submit: FS,
}

impl Default for TextInputFunctions {
  #[inline]
  fn default() -> Self { Self { on_input: TNone, on_paste: TNone, on_submit: TNone, } }
}

impl<FI, FP, FS> TextInputActions for TextInputFunctions<FI, FP, FS> {
  type ChangeOnInput<F> = TextInputFunctions<TSome<F>, FP, FS>;
  #[inline]
  fn on_input<F>(self, on_input: F) -> Self::ChangeOnInput<F> {
    TextInputFunctions { on_input: TSome(on_input), on_paste: self.on_paste, on_submit: self.on_submit }
  }
  type ChangeOnPaste<F> = TextInputFunctions<FI, TSome<F>, FS>;
  #[inline]
  fn on_paste<F>(self, on_paste: F) -> Self::ChangeOnPaste<F> {
    TextInputFunctions { on_input: self.on_input, on_paste: TSome(on_paste), on_submit: self.on_submit }
  }
  type ChangeOnSubmit<F> = TextInputFunctions<FI, FP, TSome<F>>;
  #[inline]
  fn on_submit<F>(self, on_submit: F) -> Self::ChangeOnSubmit<F> {
    TextInputFunctions { on_input: self.on_input, on_paste: self.on_paste, on_submit: TSome(on_submit) }
  }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum TextInputAction {
  Input(String),
  Paste(String),
  Submit,
}

impl<'a, S, FI, FP, FS> CreateTextInput<'a, S> for TextInputFunctions<FI, FP, FS> where
  S: State + 'a,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
  FI: TOptionFn<'a, String, S::Message> + 'a,
  FP: TOptionFn<'a, String, S::Message> + 'a,
  FS: TOptionFn<'a, (), S::Message> + 'a,
{
  type Message = TextInputAction;
  #[inline]
  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TextIn<'a, S, Self::Message>) -> TextIn<'a, S, Self::Message>,
  ) -> Elem<'a, S> {
    let mut text_input = modify(TextInput::new(placeholder, value));
    if FI::IS_SOME {
      text_input = text_input.on_input(TextInputAction::Input);
    }
    if FP::IS_SOME {
      text_input = text_input.on_paste(TextInputAction::Paste);
    }
    if FS::IS_SOME {
      text_input = text_input.on_submit(TextInputAction::Submit);
    }
    Element::new(text_input)
      .map(move |m| match m {
        TextInputAction::Input(input) => self.on_input.call(input).unwrap(),
        TextInputAction::Paste(input) => self.on_paste.call(input).unwrap(),
        TextInputAction::Submit => self.on_submit.call(()).unwrap(),
      })
  }
}

