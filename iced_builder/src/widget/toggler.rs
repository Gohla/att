use iced::{Length, Pixels};
use iced::advanced::text::{LineHeight, Renderer as TextRenderer, Shaping};
use iced::alignment::Horizontal;
use iced::widget::{Toggler, toggler};

use crate::internal::state::StateAppend;

/// Builder for a [`Toggler`] widget.
#[must_use]
pub struct TogglerBuilder<'a, S: StateAppend> where
  S::Renderer: TextRenderer,
  S::Theme: toggler::Catalog,
{
  state: S,
  toggler: Toggler<'a, S::Message, S::Theme, S::Renderer>
}

impl<'a, S: StateAppend> TogglerBuilder<'a, S> where
  S::Renderer: TextRenderer,
  S::Theme: toggler::Catalog,
{
  pub(crate) fn new(
    state: S,
    label: Option<impl Into<String>>,
    is_toggled: bool,
    toggle_fn: impl 'a + Fn(bool) -> S::Message
  ) -> Self {
    Self {
      state,
      toggler: Toggler::new(label.map(|i|i.into()), is_toggled, toggle_fn),
    }
  }


  /// Sets the size of the toggler.
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.toggler = self.toggler.size(size);
    self
  }

  /// Sets the width of the toggler's boundaries.
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.toggler = self.toggler.width(width);
    self
  }

  /// Sets the width of the toggler to [`Length::Shrink`].
  pub fn width_shrink(self) -> Self {
    self.width(Length::Shrink)
  }


  /// Sets the size of the toggler's label.
  pub fn label_size(mut self, size: impl Into<Pixels>) -> Self {
    self.toggler = self.toggler.text_size(size);
    self
  }

  /// Sets the [`LineHeight`] of the toggler's label.
  pub fn label_line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
    self.toggler = self.toggler.text_line_height(line_height);
    self
  }

  /// Sets the [`Horizontal`] alignment of the toggler's label.
  pub fn label_horizontal_alignment(mut self, alignment: Horizontal) -> Self {
    self.toggler = self.toggler.text_alignment(alignment);
    self
  }

  /// Sets the [`Shaping`] strategy of the toggler's label.
  pub fn label_shaping(mut self, shaping: Shaping) -> Self {
    self.toggler = self.toggler.text_shaping(shaping);
    self
  }

  /// Sets the [`Font`] of the toggler's label.
  ///
  /// [`Font`]: S::Renderer::Font
  pub fn label_font(mut self, font: impl Into<<S::Renderer as TextRenderer>::Font>) -> Self {
    self.toggler = self.toggler.font(font);
    self
  }

  /// Sets the spacing between the toggler and its label.
  pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
    self.toggler = self.toggler.spacing(spacing);
    self
  }


  /// Sets the `styler` function of the toggler.
  pub fn style(mut self, styler: impl Fn(&S::Theme, toggler::Status) -> toggler::Style + 'a) -> Self where
    <S::Theme as toggler::Catalog>::Class<'a>: From<toggler::StyleFn<'a, S::Theme>>
  {
    self.toggler = self.toggler.style(styler);
    self
  }

  /// Sets the `class` of the toggler.
  pub fn class(mut self, class: impl Into<<S::Theme as toggler::Catalog>::Class<'a>>) -> Self {
    self.toggler = self.toggler.class(class);
    self
  }


  /// Adds the [`Toggler`] widget to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput where
    Toggler<'a, S::Message, S::Theme, S::Renderer>: Into<S::Element>
  {
    self.state.append(self.toggler)
  }
}
