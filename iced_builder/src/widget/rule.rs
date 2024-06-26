use iced::Pixels;
use iced::widget::Rule;

use crate::internal::state::StateAppend;

/// Builder for a [`Rule`] widget.
#[must_use]
pub struct RuleBuilder<S> {
  state: S,
  width_or_height: Pixels,
  is_vertical: bool,
}

impl<'a, S: StateAppend> RuleBuilder<S> {
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      width_or_height: 1.0.into(),
      is_vertical: false,
    }
  }

  /// Makes this [`Rule`] a horizontal one of `height`.
  pub fn horizontal(mut self, height: impl Into<Pixels>) -> Self {
    self.width_or_height = height.into();
    self.is_vertical = false;
    self
  }

  /// Makes this [`Rule`] a vertical one of `width`.
  pub fn vertical(mut self, width: impl Into<Pixels>) -> Self {
    self.width_or_height = width.into();
    self.is_vertical = true;
    self
  }

  /// Adds the [`Rule`] widget to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput where
    Rule<'a>: Into<S::Element>,
  {
    let rule = if self.is_vertical {
      Rule::vertical(self.width_or_height)
    } else {
      Rule::horizontal(self.width_or_height)
    };
    self.state.append(rule)
  }
}
