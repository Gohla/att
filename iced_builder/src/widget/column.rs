use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::widget::Column;

use crate::internal::state::StateReduce;

/// Builder for a [`Column`] widget.
#[must_use]
pub struct ColumnBuilder<S> {
  state: S,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  max_width: f32,
  align_items: Alignment,
  clip: bool,
}

impl<S: StateReduce> ColumnBuilder<S> {
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      max_width: f32::INFINITY,
      align_items: Alignment::Start,
      clip: false,
    }
  }


  /// Sets the vertical spacing _between_ elements.
  ///
  /// Custom margins per element do not exist in iced. You should use this method instead! While less flexible, it helps
  /// you keep spacing between elements consistent.
  pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
    self.spacing = amount.into().0;
    self
  }

  /// Sets the [`Padding`] of the [`Column`].
  pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    self.padding = padding.into();
    self
  }


  /// Sets the width of the [`Column`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the height of the [`Column`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  /// Sets the maximum width of the [`Column`].
  pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
    self.max_width = max_width.into().0;
    self
  }

  /// Sets the width of the [`Column`] to [`Length::Fill`].
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }

  /// Sets the height of the [`Column`] to [`Length::Fill`].
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }

  /// Sets the width and height of the [`Column`] to [`Length::Fill`].
  pub fn fill(self) -> Self {
    self.fill_width().fill_height()
  }


  /// Sets the horizontal alignment of the contents of the [`Column`] .
  pub fn align_items(mut self, align: Alignment) -> Self {
    self.align_items = align;
    self
  }

  /// Sets the horizontal alignment of the contents of the [`Column`] to [`Alignment::Start`].
  pub fn align_start(self) -> Self {
    self.align_items(Alignment::Start)
  }

  /// Sets the horizontal alignment of the contents of the [`Column`] to [`Alignment::Center`].
  pub fn align_center(self) -> Self {
    self.align_items(Alignment::Center)
  }

  /// Sets the horizontal alignment of the contents of the [`Column`] to [`Alignment::End`].
  pub fn align_end(self) -> Self {
    self.align_items(Alignment::End)
  }


  /// Sets whether the contents of the [`Column`] should be clipped on overflow.
  pub fn clip(mut self, clip: bool) -> Self {
    self.clip = clip;
    self
  }


  /// Takes all current elements out of the builder, creates the [`Column`] with those elements, then adds the column to
  /// the builder and returns the builder.
  pub fn add<'a>(self) -> S::ReduceOutput where
    Vec<S::Element>: IntoIterator<Item=Element<'a, S::Message, S::Theme, S::Renderer>>, // For `Column::with_children`
    Column<'a, S::Message, S::Theme, S::Renderer>: Into<S::Element>, // For `.into()`
  { // Can't use `Elem<'a, S>` in above bounds due to it crashing RustRover.
    self.state.reduce(|vec| {
      // TODO: use `from_vec`, but need to figure out how add a bound that `vec` is a `Vec<Element<...>>`.
      Column::with_children(vec)
        .spacing(self.spacing)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .max_width(self.max_width)
        .align_items(self.align_items)
        .clip(self.clip)
        .into()
    })
  }
}
