//! Heap-allocated list:
//!
//! - Limited compile-time type safety, checks required at run-time.
//! - Some run-time overhead. TODO: benchmark this
//! - Low compile-time overhead.
//! - Every operation is type-preserving.

use iced::advanced::Renderer;
use iced::Element;

use super::{Elem, StateAdd, StateConsume, StateMap, StateTake, StateTakeAll, StateTypes};
use super::super::WidgetBuilder;

/// Heap-allocated list.
pub enum HeapList<E> {
  Zero,
  One(E, usize),
  Many(Vec<E>),
}

impl<E> Default for HeapList<E> {
  #[inline]
  fn default() -> Self { Self::Zero }
}

impl<E> HeapList<E> {
  #[inline]
  pub fn with_capacity(capacity: usize) -> Self { Self::Many(Vec::with_capacity(capacity)) }

  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    match self {
      HeapList::Zero => *self = HeapList::Many(Vec::with_capacity(additional)),
      HeapList::One(_, reserve_additional) => *reserve_additional += additional,
      HeapList::Many(ref mut vec) => vec.reserve(additional),
    }
  }

  #[inline]
  fn add(self, new_element: E) -> Self {
    match self {
      HeapList::Zero => HeapList::One(new_element, 0),
      HeapList::One(element, reserve_additional) => {
        let vec = if reserve_additional > 0 {
          let mut vec = Vec::with_capacity(2 + reserve_additional);
          vec.push(element);
          vec.push(new_element);
          vec
        } else {
          vec![element, new_element]
        };
        HeapList::Many(vec)
      },
      HeapList::Many(mut vec) => {
        vec.push(new_element);
        HeapList::Many(vec)
      },
    }
  }
  #[inline]
  fn unwrap(self) -> (Vec<E>, usize) {
    match self {
      HeapList::Zero => (vec![], 0),
      HeapList::One(element, reserve_additional) => (vec![element], reserve_additional),
      HeapList::Many(vec) => (vec, 0),
    }
  }
}


// Implement state traits for `HeapList`.

impl<'a, E> StateTypes for HeapList<E> where
  E: Elem
{
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E> StateAdd for HeapList<E> where
  E: Elem
{

  type AddOutput = WidgetBuilder<Self>;
  #[inline]
  fn add<I: Into<Self::Element>>(self, into_elem: I) -> Self::AddOutput {
    WidgetBuilder(self.add(into_elem.into()))
  }
}

impl<'a, M, T, R> StateConsume<'a> for HeapList<Element<'a, M, T, R>> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
{
  type ConsumeOutput = WidgetBuilder<Self>;
  fn consume<F: FnOnce(Vec<Element<'a, M, T, R>>) -> Element<'a, M, T, R>>(self, produce: F) -> Self::ConsumeOutput {
    let (vec, reserve_additional) = self.unwrap();
    let element = produce(vec);
    WidgetBuilder(HeapList::One(element, reserve_additional))
  }
}

impl<'a, M, T, R> StateMap<'a> for HeapList<Element<'a, M, T, R>> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
{
  type MapOutput = WidgetBuilder<Self>;
  #[inline]
  fn map_last<F: FnOnce(Element<'a, M, T, R>) -> Element<'a, M, T, R>>(self, map: F) -> Self::MapOutput {
    let mapped = match self {
      HeapList::Zero => panic!("builder should have at least 1 element"),
      HeapList::One(element, reserve_additional) => HeapList::One(map(element), reserve_additional),
      HeapList::Many(mut vec) => {
        let element = vec.pop()
          .unwrap_or_else(|| panic!("builder should have at least 1 element"));
        let element = map(element);
        vec.push(element);
        HeapList::Many(vec)
      }
    };
    WidgetBuilder(mapped)
  }
}

impl<'a, M, T, R> StateTakeAll<'a> for HeapList<Element<'a, M, T, R>> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
{
  #[inline]
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>> {
    self.unwrap().0
  }
}

impl<'a, M, T, R> StateTake<'a> for HeapList<Element<'a, M, T, R>> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
{
  #[inline]
  fn take(self) -> Element<'a, M, T, R> {
    match self {
      HeapList::Zero => panic!("builder should have precisely 1 element, but it has 0"),
      HeapList::One(element, _) => element,
      HeapList::Many(mut vec) => {
        let len = vec.len();
        let 1 = len else {
          panic!("builder should have precisely 1 element, but it has {}", len);
        };
        vec.drain(..).next().unwrap()
      }
    }
  }
}
