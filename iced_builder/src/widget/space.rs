use iced::Length;
use iced::widget::Space;

use crate::internal::state::StateAppend;

/// Builder for a [`Space`] widget.
#[must_use]
pub struct SpaceBuilder<S> {
  state: S,
  width: Length,
  height: Length,
}
impl<S: StateAppend> SpaceBuilder<S> {
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      width: Length::Shrink,
      height: Length::Shrink,
    }
  }


  /// Sets the `width` of the [`Space`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the `height` of the [`Space`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  /// Sets the `width` of the [`Space`] to `Length::Fill`.
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }

  /// Sets the `height` of the [`Space`] to `Length::Fill`.
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }


  /// Adds the [`Space`] widget to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput where
    Space: Into<S::Element>,
  {
    let space = Space::new(self.width, self.height);
    self.state.append(space)
  }
}
