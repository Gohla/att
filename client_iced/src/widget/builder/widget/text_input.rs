use iced::advanced::text::Renderer as TextRenderer;
use iced::Element;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use iced::widget::TextInput;

use crate::widget::builder::state::Elem;

use super::super::state::State;

pub trait TextInputActions<'a, M> {
  type ChangeOnInput<F> where F: 'a;
  fn on_input<F: Fn(String) -> M + 'a>(self, on_input: F) -> Self::ChangeOnInput<F>;

  type ChangeOnPaste<F> where F: 'a;
  fn on_paste<F: Fn(String) -> M + 'a>(self, on_paste: F) -> Self::ChangeOnPaste<F>;

  type ChangeOnSubmit<F> where F: 'a;
  fn on_submit<F: Fn() -> M + 'a>(self, on_submit: F) -> Self::ChangeOnSubmit<F>;
}

type TextIn<'a, S, C> = TextInput<'a, <C as CreateTextInput<'a, S>>::Message, <S as State>::Theme, <S as State>::Renderer>;

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
    modify: impl FnOnce(TextIn<'a, S, Self>) -> TextIn<'a, S, Self>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer>;
}

/// Passthrough which does not modify the message type, thus the message type must implement [`Clone`].
pub struct TextInputPassthrough;
impl<'a, M> TextInputActions<'a, M> for TextInputPassthrough {
  type ChangeOnInput<F: 'a> = <TextInputFunctions as TextInputActions<'a, M>>::ChangeOnInput<F>;
  #[inline]
  fn on_input<F: Fn(String) -> M + 'a>(self, on_input: F) -> Self::ChangeOnInput<F> {
    TextInputFunctions::default().on_input(on_input)
  }

  type ChangeOnPaste<F: 'a> = <TextInputFunctions as TextInputActions<'a, M>>::ChangeOnPaste<F>;
  #[inline]
  fn on_paste<F: Fn(String) -> M + 'a>(self, on_paste: F) -> Self::ChangeOnPaste<F> {
    TextInputFunctions::default().on_paste(on_paste)
  }

  type ChangeOnSubmit<F: 'a> = <TextInputFunctions as TextInputActions<'a, M>>::ChangeOnSubmit<F>;
  #[inline]
  fn on_submit<F: Fn() -> M + 'a>(self, on_submit: F) -> Self::ChangeOnSubmit<F> {
    TextInputFunctions::default().on_submit(on_submit)
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
    modify: impl FnOnce(TextIn<'a, S, Self>) -> TextIn<'a, S, Self>,
  ) -> Elem<'a, S> {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    Element::new(text_input)
  }
}

/// Modify message type to [`TextInputAction`] which is [`Clone`], without our callbacks needing to implement clone.
pub struct TextInputFunctions<FI = (), FP = (), FS = ()> {
  on_input: FI,
  on_paste: FP,
  on_submit: FS,
}
impl Default for TextInputFunctions {
  #[inline]
  fn default() -> Self { Self { on_input: (), on_paste: (), on_submit: (), } }
}

pub struct Fn1<F>(F);
pub struct Fn0<F>(F);

impl<'a, M, FI, FP, FS> TextInputActions<'a, M> for TextInputFunctions<FI, FP, FS> {
  type ChangeOnInput<F: 'a> = TextInputFunctions<Fn1<F>, FP, FS>;
  fn on_input<F: Fn(String) -> M + 'a>(self, on_input: F) -> Self::ChangeOnInput<F> {
    TextInputFunctions { on_input: Fn1(on_input), on_paste: self.on_paste, on_submit: self.on_submit }
  }

  type ChangeOnPaste<F: 'a> = TextInputFunctions<FI, Fn1<F>, FS>;
  fn on_paste<F: Fn(String) -> M + 'a>(self, on_paste: F) -> Self::ChangeOnPaste<F> {
    TextInputFunctions { on_input: self.on_input, on_paste: Fn1(on_paste), on_submit: self.on_submit }
  }

  type ChangeOnSubmit<F: 'a> = TextInputFunctions<FI, FP, Fn0<F>>;
  fn on_submit<F: Fn() -> M + 'a>(self, on_submit: F) -> Self::ChangeOnSubmit<F> {
    TextInputFunctions { on_input: self.on_input, on_paste: self.on_paste, on_submit: Fn0(on_submit) }
  }
}

trait Callback<I, M> {
  fn should_register() -> bool;
  fn call(&self, input: I) -> Option<M>;
}
impl<M, I> Callback<I, M> for () {
  #[inline]
  fn should_register() -> bool { false }
  #[inline]
  fn call(&self, _input: I) -> Option<M> { None }
}
impl<'a, I, M, F: Fn(I) -> M + 'a> Callback<I, M> for Fn1<F> {
  #[inline]
  fn should_register() -> bool { true }
  #[inline]
  fn call(&self, input: I) -> Option<M> { Some(self.0(input)) }
}
impl<'a, M, F: Fn() -> M + 'a> Callback<(), M> for Fn0<F> {
  #[inline]
  fn should_register() -> bool { true }
  #[inline]
  fn call(&self, _input: ()) -> Option<M> { Some(self.0()) }
}

#[derive(Clone)]
pub enum TextInputAction {
  Input(String),
  Paste(String),
  Submit,
}

impl<'a, S, FI, FP, FS> CreateTextInput<'a, S> for TextInputFunctions<FI, FP, FS> where
  S: State + 'a,
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
  FI: Callback<String, S::Message> + 'a,
  FP: Callback<String, S::Message> + 'a,
  FS: Callback<(), S::Message> + 'a,
{
  type Message = TextInputAction;

  #[inline]
  fn create(
    self,
    placeholder: &str,
    value: &str,
    modify: impl FnOnce(TextIn<'a, S, Self>) -> TextIn<'a, S, Self>,
  ) -> Elem<'a, S> {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    if FI::should_register() {
      text_input = text_input.on_input(TextInputAction::Input);
    }
    if FP::should_register() {
      text_input = text_input.on_paste(TextInputAction::Paste);
    }
    if FS::should_register() {
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

