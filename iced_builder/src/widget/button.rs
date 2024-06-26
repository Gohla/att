use iced::{Length, Padding};
use iced::widget::button;

use crate::internal::state::{Elem, ElemM, State, StateAppend};
use crate::internal::widget::button::{ButtonActions, ButtonPassthrough, CreateButton};

/// Builder for a [`Button`](button::Button) widget.
#[must_use]
pub struct ButtonBuilder<'a, S: State, C, A = ButtonPassthrough> where
  S::Theme: button::Catalog
{
  state: S,
  content: C,
  actions: A,
  disabled: bool,
  width: Length,
  height: Length,
  padding: Padding,
  class: <S::Theme as button::Catalog>::Class<'a>,
}

impl<'a, S: State, C> ButtonBuilder<'a, S, C> where
  S::Theme: button::Catalog,
{
  pub(crate) fn new(state: S, content: C) -> Self {
    Self {
      state,
      content,
      actions: ButtonPassthrough,
      disabled: false,
      width: Length::Shrink,
      height: Length::Shrink,
      padding: 5.0.into(),
      class: <S::Theme as button::Catalog>::default(),
    }
  }
}

impl<'a, S: State, C, A: ButtonActions> ButtonBuilder<'a, S, C, A> where
  S::Theme: button::Catalog
{
  /// Sets the width.
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the height.
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  /// Sets the [`Padding`].
  pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    self.padding = padding.into();
    self
  }


  /// Sets the function that will be called when the button is pressed to `on_press`.
  ///
  /// If this method is not called, the button will be disabled.
  pub fn on_press<F: Fn() -> S::Message + 'a>(self, on_press: F) -> ButtonBuilder<'a, S, C, A::ChangeOnPress<F>> {
    self.replace_actions(|actions| actions.on_press(on_press))
  }

  /// Sets whether the button is `disabled`.
  pub fn disabled(mut self, disabled: bool) -> Self {
    self.disabled = disabled;
    self
  }


  /// Sets the `styler` function.
  pub fn style(mut self, styler: impl Fn(&S::Theme, button::Status) -> button::Style + 'a) -> Self where
    <S::Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, S::Theme>>,
  {
    self.class = (Box::new(styler) as button::StyleFn<'a, S::Theme>).into();
    self
  }

  /// Sets the style to [`button::primary`].
  ///
  /// Only available when the theme is the built-in [`iced::Theme`].
  pub fn primary_style(self) -> Self where
    S: State<Theme=iced::Theme>
  {
    self.style(button::primary)
  }

  /// Sets the style to [`button::secondary`].
  ///
  /// Only available when the theme is the built-in [`iced::Theme`].
  pub fn secondary_style(self) -> Self where
    S: State<Theme=iced::Theme>
  {
    self.style(button::secondary)
  }

  /// Sets the style to [`button::success`].
  ///
  /// Only available when the theme is the built-in [`iced::Theme`].
  pub fn success_style(self) -> Self where
    S: State<Theme=iced::Theme>
  {
    self.style(button::success)
  }

  /// Sets the style to [`button::danger`].
  ///
  /// Only available when the theme is the built-in [`iced::Theme`].
  pub fn danger_style(self) -> Self where
    S: State<Theme=iced::Theme>
  {
    self.style(button::danger)
  }

  /// Sets the style to [`button::text`].
  ///
  /// Only available when the theme is the built-in [`iced::Theme`].
  pub fn text_style(self) -> Self where
    S: State<Theme=iced::Theme>
  {
    self.style(button::text)
  }

  /// Sets the `class`.
  pub fn class(mut self, class: impl Into<<S::Theme as button::Catalog>::Class<'a>>) -> Self {
    self.class = class.into();
    self
  }


  fn replace_actions<AA>(self, change: impl FnOnce(A) -> AA) -> ButtonBuilder<'a, S, C, AA> {
    ButtonBuilder {
      state: self.state,
      content: self.content,
      actions: change(self.actions),
      disabled: self.disabled,
      width: self.width,
      height: self.height,
      padding: self.padding,
      class: self.class,
    }
  }
}

impl<'a, S: StateAppend, C, A: CreateButton<'a, S>> ButtonBuilder<'a, S, C, A> where
  S::Theme: button::Catalog,
  Elem<'a, S>: Into<S::Element>,
  C: Into<ElemM<'a, S, A::Message>>,
{
  /// Adds the [`Button`](button::Button) to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput {
    let element = self.actions.create(self.content, |button| {
      let mut button = button
        .width(self.width)
        .height(self.height)
        .padding(self.padding)
        .class(self.class);
      if self.disabled {
        button = button.on_press_maybe(None);
      }
      button
    });
    self.state.append(element)
  }
}
