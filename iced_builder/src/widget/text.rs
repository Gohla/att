use std::borrow::Cow;

use iced::{Color, Length, Pixels};
use iced::advanced::text::{LineHeight, Renderer as TextRenderer, Shaping};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Text, text};

use crate::internal::state::StateAppend;

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<'a, S: StateAppend> where
  S::Renderer: TextRenderer,
  S::Theme: text::Catalog,
{
  state: S,
  text: Text<'a, S::Theme, S::Renderer>
}

impl<'a, S: StateAppend> TextBuilder<'a, S> where
  S::Renderer: TextRenderer,
  S::Theme: text::Catalog,
{
  pub(crate) fn new(state: S, content: Cow<'a, str>) -> Self { // TODO: change to impl IntoFragment<'a>
    Self {
      state,
      text: Text::new(content),
    }
  }


  /// Sets the size of the [`Text`].
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.text = self.text.size(size);
    self
  }

  /// Sets the [`LineHeight`] of the [`Text`].
  pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
    self.text = self.text.line_height(line_height);
    self
  }

  /// Sets the [`Font`] of the [`Text`].
  ///
  /// [`Font`]: S::Renderer::Font
  pub fn font(mut self, font: impl Into<<S::Renderer as TextRenderer>::Font>) -> Self {
    self.text = self.text.font(font);
    self
  }

  /// Sets the width of the [`Text`] boundaries.
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.text = self.text.width(width);
    self
  }

  /// Sets the height of the [`Text`] boundaries.
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.text = self.text.height(height);
    self
  }

  /// Sets the [`Horizontal`] alignment of the [`Text`].
  pub fn horizontal_alignment(mut self, alignment: Horizontal) -> Self {
    self.text = self.text.horizontal_alignment(alignment);
    self
  }

  /// Sets the [`Vertical`] alignment of the [`Text`].
  pub fn vertical_alignment(mut self, alignment: Vertical) -> Self {
    self.text = self.text.vertical_alignment(alignment);
    self
  }

  /// Sets the [`Shaping`] strategy of the [`Text`].
  pub fn shaping(mut self, shaping: Shaping) -> Self {
    self.text = self.text.shaping(shaping);
    self
  }


  /// Sets the `styler` function of the [`Text`].
  pub fn style(mut self, styler: impl Fn(&S::Theme) -> text::Style + 'a) -> Self where
    <S::Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, S::Theme>>
  {
    self.text = self.text.style(styler);
    self
  }

  /// Sets a [`Color`] as the style of the [`Text`].
  pub fn color(mut self, color: impl Into<Color>) -> Self where
    <S::Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, S::Theme>>
  {
    self.text = self.text.color(color);
    self
  }

  /// Sets the `class` of the [`Text`].
  pub fn class(mut self, class: impl Into<<S::Theme as text::Catalog>::Class<'a>>) -> Self {
    self.text = self.text.class(class);
    self
  }


  /// Adds the [`Text`] widget to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput where
    Text<'a, S::Theme, S::Renderer>: Into<S::Element>
  {
    self.state.append(self.text)
  }
}
