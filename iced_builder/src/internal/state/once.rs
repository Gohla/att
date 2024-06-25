use std::marker::PhantomData;

use super::{El, State, StateAppend};

impl<E: El> State for PhantomData<E> {
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E: El> StateAppend for PhantomData<E> {
  type AddOutput = E;
  #[inline]
  fn append(self, into_element: impl Into<Self::Element>) -> Self::AddOutput { into_element.into() }
}
