use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::widget::Row;

use crate::internal::state::StateReduce;

/// Builder for a [`Row`] widget.
#[must_use]
pub struct RowBuilder<S> {
  state: S,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
  clip: bool,
}
impl<S: StateReduce> RowBuilder<S> {
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      align_items: Alignment::Start,
      clip: false,
    }
  }


  /// Sets the horizontal spacing _between_ elements.
  ///
  /// Custom margins per element do not exist in iced. You should use this method instead! While less flexible, it helps
  /// you keep spacing between elements consistent.
  pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
    self.spacing = spacing.into().0;
    self
  }

  /// Sets the [`Padding`] of the [`Row`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }

  /// Sets the width of the [`Row`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the height of the [`Row`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  /// Sets the width of the [`Row`] to [`Length::Fill`].
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }

  /// Sets the height of the [`Row`] to [`Length::Fill`].
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }

  /// Sets the width and height of the [`Row`] to [`Length::Fill`].
  pub fn fill(self) -> Self {
    self.fill_width().fill_height()
  }


  /// Sets the vertical alignment of the contents of the [`Row`].
  pub fn align_items(mut self, align: Alignment) -> Self {
    self.align_items = align;
    self
  }

  /// Sets the vertical alignment of the contents of the [`Row`] to [`Alignment::Start`].
  pub fn align_start(self) -> Self {
    self.align_items(Alignment::Start)
  }

  /// Sets the vertical alignment of the contents of the [`Row`] to [`Alignment::Center`].
  pub fn align_center(self) -> Self {
    self.align_items(Alignment::Center)
  }

  /// Sets the vertical alignment of the contents of the [`Row`] to [`Alignment::End`].
  pub fn align_end(self) -> Self {
    self.align_items(Alignment::End)
  }


  /// Sets whether the contents of the [`Row`] should be clipped on overflow.
  pub fn clip(mut self, clip: bool) -> Self {
    self.clip = clip;
    self
  }


  /// Takes all current elements out of the builder, creates the [`Row`] with those elements, then adds the row to
  /// the builder and returns the builder.
  pub fn add<'a>(self) -> S::ReduceOutput where
    Vec<S::Element>: IntoIterator<Item=Element<'a, S::Message, S::Theme, S::Renderer>>, // For `Row::with_children`
    Row<'a, S::Message, S::Theme, S::Renderer>: Into<S::Element>, // For `.into()`
  { // Can't use `Elem<'a, S>` in above bounds due to it crashing RustRover.
    self.state.reduce(|vec| {
      // TODO: use `from_vec`, but need to figure out how add a bound that `vec` is a `Vec<Element<...>>`.
      Row::with_children(vec)
        .spacing(self.spacing)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .align_items(self.align_items)
        .clip(self.clip)
        .into()
    })
  }
}
