//! Heap implementation: run-time type safety, not zero-cost, but type does not change.

use iced::advanced::Renderer;
use iced::Element;

use super::{StateAdd, StateConsume, StateMap, StateTake, StateTakeAll, Types};
use super::super::WidgetBuilder;

/// Heap-based list consisting either of a `Vec` with any number of elements, or a single element (with optional
/// reserve_additional)
pub enum HeapList<E> {
  Any(Vec<E>),
  One(E, usize),
}
impl<E> Default for HeapList<E> {
  #[inline]
  fn default() -> Self {
    Self::Any(Vec::new()) // TODO: prevent creation of empty vec with a Zero variant?
  }
}

impl<E> HeapList<E> {
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
  fn add(self, new_element: E) -> Self {
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
}


// Implement state traits for `HeapList`.

impl<'a, M: 'a, R: Renderer + 'a> Types<'a> for HeapList<Element<'a, M, R>> {
  type Message = M;
  type Renderer = R;
  type Theme = R::Theme;
}

impl<'a, M: 'a, R: Renderer + 'a> StateAdd<'a> for HeapList<Element<'a, M, R>> {
  type AddOutput = WidgetBuilder<Self>;
  #[inline]
  fn add(self, element: Element<'a, M, R>) -> Self::AddOutput {
    WidgetBuilder(self.add(element))
  }
}

impl<'a, M: 'a, R: Renderer + 'a> StateConsume<'a> for HeapList<Element<'a, M, R>> {
  type ConsumeOutput = WidgetBuilder<Self>;
  fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeOutput {
    let vec = self.to_vec();
    let element = produce(vec);
    WidgetBuilder(HeapList::One(element, 0))
  }
}

impl<'a, M: 'a, R: Renderer + 'a> StateMap<'a> for HeapList<Element<'a, M, R>> {
  type MapOutput = WidgetBuilder<Self>;
  #[inline]
  fn map_last<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapOutput {
    let mapped = match self {
      HeapList::Any(mut vec) => {
        let element = vec.pop()
          .unwrap_or_else(|| panic!("builder should have at least 1 element"));
        let element = map(element);
        vec.push(element);
        HeapList::Any(vec)
      }
      HeapList::One(element, reserve_additional) => HeapList::One(map(element), reserve_additional),
    };
    WidgetBuilder(mapped)
  }
}

impl<'a, M: 'a, R: Renderer + 'a> StateTakeAll<'a> for HeapList<Element<'a, M, R>> {
  #[inline]
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    self.to_vec()
  }
}

impl<'a, M: 'a, R: Renderer + 'a> StateTake<'a> for HeapList<Element<'a, M, R>> {
  #[inline]
  fn take(self) -> Element<'a, M, R> {
    match self {
      HeapList::Any(mut vec) => {
        let len = vec.len();
        vec.drain(..).next()
          .unwrap_or_else(|| panic!("builder should have precisely 1 element, but it has {}", len))
      }
      HeapList::One(element, _) => element,
    }
  }
}
