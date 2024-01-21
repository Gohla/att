use std::marker::PhantomData;

use iced::advanced::Renderer;
use iced::Element;

use super::{StateAdd, StateTypes};

impl<'a, M: 'a, R: Renderer + 'a> StateTypes<'a> for PhantomData<Element<'a, M, R>> {
  type Message = M;
  type Renderer = R;
  type Theme = R::Theme;
}

impl<'a, M: 'a, R: Renderer + 'a> StateAdd<'a> for PhantomData<Element<'a, M, R>> {
  type AddOutput = Element<'a, M, R>;
  #[inline]
  fn add(self, element: Element<'a, M, R>) -> Self::AddOutput {
    element
  }
}
