//! Heap implementation: run-time type safety, not zero-cost, but type does not change.

use iced::advanced::Renderer;
use iced::Element;

use crate::widget::builder::WidgetBuilder;

use super::{AnyState, OneState, Types};

/// Heap-based list consisting either of a `Vec` with any number of elements, or a single element (with optional
/// reserve_additional)
pub enum HeapList<E> {
  Any(Vec<E>),
  One(E, usize),
}
impl<E> HeapList<E> {
  #[inline]
  pub fn new() -> Self { Self::Any(Vec::new()) }
  #[inline]
  pub fn with_capacity(capacity: usize) -> Self { Self::Any(Vec::with_capacity(capacity)) }

  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    match self {
      HeapList::Any(ref mut vec) => vec.reserve(additional),
      HeapList::One(_, reserve_additional) => *reserve_additional += additional,
    }
  }

  #[inline]
  fn push(self, new_element: E) -> Self {
    match self {
      HeapList::Any(mut vec) => {
        vec.push(new_element);
        HeapList::Any(vec)
      },
      HeapList::One(element, reserve_additional) => {
        let vec = if reserve_additional > 0 {
          let mut vec = Vec::with_capacity(2 + reserve_additional);
          vec.push(element);
          vec.push(new_element);
          vec
        } else {
          vec![element, new_element]
        };
        HeapList::Any(vec)
      },
    }
  }
  #[inline]
  fn to_vec(self) -> Vec<E> {
    match self {
      HeapList::Any(vec) => vec,
      HeapList::One(element, _) => vec![element], // Note: ignore reserve_additional, since the vec will be consumed as-is.
    }
  }
  #[inline]
  fn take(self) -> (E, usize) {
    match self {
      HeapList::Any(mut vec) => {
        let len = vec.len();
        let 1 = len else {
          panic!("builder should have precisely 1 element, but it has {}", len);
        };
        let element = vec.drain(..).next().unwrap();
        (element, 0)
      }
      HeapList::One(element, reserve_additional) => (element, reserve_additional),
    }
  }
}

// Implement internal traits for `HeapList`.
impl<'a, M, R> Types<'a> for HeapList<Element<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a,
{
  type Message = M;
  type Renderer = R;
  type Theme = R::Theme;
}
impl<'a, M, R> AnyState<'a> for HeapList<Element<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a,
{
  type AddBuilder = WidgetBuilder<Self>;
  #[inline]
  fn add(self, element: Element<'a, M, R>) -> Self::AddBuilder {
    let heap = self.push(element);
    WidgetBuilder(heap)
  }

  type ConsumeBuilder = WidgetBuilder<Self>;
  #[inline]
  fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeBuilder {
    let vec = self.to_vec();
    let element = produce(vec);
    WidgetBuilder(HeapList::One(element, 0))
  }

  #[inline]
  fn take_vec(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    self.to_vec()
  }
}
impl<'a, M, R> OneState<'a> for HeapList<Element<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a
{
  type MapBuilder = WidgetBuilder<Self>;
  #[inline]
  fn map<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapBuilder {
    let (element, reserve_additional) = self.take();
    let element = map(element);
    WidgetBuilder(HeapList::One(element, reserve_additional))
  }

  #[inline]
  fn take(self) -> Element<'a, M, R> {
    self.take().0
  }
}
