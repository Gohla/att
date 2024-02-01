use std::marker::PhantomData;

use super::{Elem, StateAdd, StateTypes};

impl<'a, E> StateTypes for PhantomData<E> where
  E: Elem
{
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E> StateAdd for PhantomData<E> where
  E: Elem
{

  type AddOutput = E;
  #[inline]
  fn add<I: Into<Self::Element>>(self, into_elem: I) -> Self::AddOutput {
    into_elem.into()
  }
}
