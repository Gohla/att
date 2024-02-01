use std::marker::PhantomData;

use super::{Elem, StateAdd, StateTypes};

impl<'a, E> StateTypes<'a> for PhantomData<E> where
  E: Elem<'a>
{
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<'a, E> StateAdd<'a> for PhantomData<E> where
  E: Elem<'a>
{
  type Element = E;
  type AddOutput = E;
  #[inline]
  fn add(self, element: E) -> Self::AddOutput {
    element
  }
}
