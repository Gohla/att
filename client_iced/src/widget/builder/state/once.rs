use std::marker::PhantomData;

use iced::advanced::Renderer;
use iced::Element;

use super::{StateAdd, StateTypes};

impl<'a, M, T, R> StateTypes<'a> for PhantomData<Element<'a, M, T, R>> where
  M: 'a,
  T: 'a,
  R: Renderer + 'a
{
  type Message = M;
  type Theme = T;
  type Renderer = R;
}

impl<'a, M, T, R> StateAdd<'a> for PhantomData<Element<'a, M, T, R>> where
  M: 'a,
  T: 'a,
  R: Renderer + 'a
{
  type AddOutput = Element<'a, M, T, R>;
  #[inline]
  fn add(self, element: Element<'a, M, T, R>) -> Self::AddOutput {
    element
  }
}
