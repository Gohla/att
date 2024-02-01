use std::marker::PhantomData;

use super::{Elem, State, StateAdd};

impl<E: Elem> State for PhantomData<E> {
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E: Elem> StateAdd for PhantomData<E> {
  type AddOutput = E;
  #[inline]
  fn add(self, into: impl Into<Self::Element>) -> Self::AddOutput {
    into.into()
  }
}
