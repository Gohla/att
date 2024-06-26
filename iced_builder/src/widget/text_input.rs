use iced::{Length, Padding, Pixels};
use iced::advanced::text::{LineHeight, Renderer as TextRenderer};
use iced::widget::text_input;
use iced::widget::text_input::Status;

use crate::internal::state::{Elem, StateAppend};
use crate::internal::widget::text_input::{CreateTextInput, TextInputActions, TextInputPassthrough};

/// Builder for a [`TextInput`](text_input::TextInput) widget.
#[must_use]
pub struct TextInputBuilder<'a, S: StateAppend, A = TextInputPassthrough> where
  S::Renderer: TextRenderer,
  S::Theme: text_input::Catalog,
{
  state: S,
  id: Option<text_input::Id>,
  placeholder: &'a str,
  value: &'a str,
  secure: bool,
  font: Option<<S::Renderer as TextRenderer>::Font>,
  width: Length,
  padding: Padding,
  size: Option<Pixels>,
  line_height: LineHeight,
  actions: A,
  icon: Option<text_input::Icon<<S::Renderer as TextRenderer>::Font>>,
  class: <S::Theme as text_input::Catalog>::Class<'a>,
}

impl<'a, S: StateAppend> TextInputBuilder<'a, S> where
  S::Renderer: TextRenderer,
  S::Theme: text_input::Catalog
{
  pub(crate) fn new(state: S, placeholder: &'a str, value: &'a str) -> Self {
    Self {
      state,
      id: None,
      placeholder,
      value,
      secure: false,
      font: None,
      width: Length::Fill,
      padding: Padding::new(5.0),
      size: None,
      line_height: LineHeight::default(),
      actions: TextInputPassthrough,
      icon: None,
      class: <S::Theme as text_input::Catalog>::default(),
    }
  }
}

impl<'a, S: StateAppend, A: TextInputActions> TextInputBuilder<'a, S, A> where
  S::Renderer: TextRenderer,
  S::Theme: text_input::Catalog
{
  /// Sets the [`text_input::Id`].
  pub fn id(mut self, id: text_input::Id) -> Self {
    self.id = Some(id);
    self
  }

  /// Converts this into a secure password input.
  pub fn secure(mut self) -> Self {
    self.secure = true;
    self
  }

  /// Sets the [`Font`].
  ///
  /// [`Font`]: S::Renderer::Font
  pub fn font(mut self, font: <S::Renderer as TextRenderer>::Font) -> Self {
    self.font = Some(font);
    self
  }

  /// Sets the [`text_input::Icon`].
  pub fn icon(mut self, icon: text_input::Icon<<S::Renderer as TextRenderer>::Font>) -> Self {
    self.icon = Some(icon);
    self
  }

  /// Sets the width.
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the [`Padding`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }

  /// Sets the text size.
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }

  /// Sets the [`LineHeight`].
  pub fn line_height(
    mut self,
    line_height: impl Into<LineHeight>,
  ) -> Self {
    self.line_height = line_height.into();
    self
  }


  /// Sets the function that will be called when text is typed into the text input to `on_input`.
  ///
  /// If this method is not called, the [`TextInput`] will be disabled.
  pub fn on_input<F: Fn(String) -> S::Message + 'a>(self, on_input: F) -> TextInputBuilder<'a, S, A::ChangeOnInput<F>> {
    self.replace_actions(|actions| actions.on_input(on_input))
  }

  /// Sets the function that will be called when text is pasted into the text input to `on_paste`.
  pub fn on_paste<F: Fn(String) -> S::Message + 'a>(self, on_paste: F) -> TextInputBuilder<'a, S, A::ChangeOnPaste<F>> {
    self.replace_actions(|actions| actions.on_paste(on_paste))
  }

  /// Sets the function that will be called when the text input is focussed and the enter key is pressed to
  /// `on_paste`.
  pub fn on_submit<F: Fn() -> S::Message + 'a>(self, on_submit: F) -> TextInputBuilder<'a, S, A::ChangeOnSubmit<F>> {
    self.replace_actions(|actions| actions.on_submit(on_submit))
  }


  /// Sets the `styler` function.
  pub fn style(mut self, styler: impl Fn(&S::Theme, Status) -> text_input::Style + 'a) -> Self where
    <S::Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, S::Theme>>
  {
    self.class = (Box::new(styler) as text_input::StyleFn<'a, S::Theme>).into();
    self
  }

  /// Sets the `class`.
  pub fn class(mut self, class: impl Into<<S::Theme as text_input::Catalog>::Class<'a>>) -> Self {
    self.class = class.into();
    self
  }


  fn replace_actions<AA>(self, change: impl FnOnce(A) -> AA) -> TextInputBuilder<'a, S, AA> {
    TextInputBuilder {
      state: self.state,
      id: self.id,
      placeholder: self.placeholder,
      value: self.value,
      secure: self.secure,
      font: self.font,
      width: self.width,
      padding: self.padding,
      size: self.size,
      line_height: self.line_height,
      actions: change(self.actions),
      icon: self.icon,
      class: self.class
    }
  }
}

impl<'a, S: StateAppend, A: CreateTextInput<'a, S>> TextInputBuilder<'a, S, A> where
  S::Renderer: TextRenderer,
  S::Theme: text_input::Catalog,
  Elem<'a, S>: Into<S::Element>,
{
  /// Adds the [`TextInput`](text_input::TextInput) to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput {
    let element = self.actions.create(&self.placeholder, &self.value, |mut text_input| {
      if let Some(id) = self.id {
        text_input = text_input.id(id);
      }
      if let Some(font) = self.font {
        text_input = text_input.font(font);
      }
      if let Some(size) = self.size {
        text_input = text_input.size(size);
      }
      if let Some(icon) = self.icon {
        text_input = text_input.icon(icon);
      }
      text_input
        .secure(self.secure)
        .width(self.width)
        .padding(self.padding)
        .line_height(self.line_height)
        .class(self.class)
    });
    self.state.append(element)
  }
}
