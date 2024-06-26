use iced::Element;

use crate::internal::state::{Elem, ElemM, StateAppend};

/// Builder for an [`Element`].
#[must_use]
pub struct ElementBuilder<'a, S: StateAppend, M> {
  state: S,
  element: ElemM<'a, S, M>,
}
impl<'a, S: StateAppend, M> ElementBuilder<'a, S, M> {
  pub(crate) fn new(state: S, element: Element<'a, M, S::Theme, S::Renderer>) -> Self {
    Self { state, element }
  }

  /// Applies a transformation to the produced message of the [`Element`].
  pub fn map(self, f: impl Fn(M) -> S::Message + 'a) -> ElementBuilder<'a, S, S::Message> where
    M: 'a,
    S: 'a,
  {
    let element = self.element.map(f);
    ElementBuilder { state: self.state, element }
  }
}

impl<'a, S: StateAppend> ElementBuilder<'a, S, S::Message> where
  Elem<'a, S>: Into<S::Element>,
{
  /// Adds the [`Element`] to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput {
    self.state.append(self.element)
  }
}
