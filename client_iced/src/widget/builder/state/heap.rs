//! Heap-allocated list:
//!
//! - Limited compile-time type safety, checks required at run-time.
//! - Some run-time overhead. TODO: benchmark this
//! - Low compile-time overhead.
//! - Every operation is type-preserving.

use super::{El, State, StateAppend, StateMap, StateReduce, StateTake, StateTakeAll};
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

impl<E: El> State for HeapList<E> {
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E: El> StateAppend for HeapList<E> {
  type AddOutput = WidgetBuilder<Self>;
  #[inline]
  fn append(self, into_element: impl Into<Self::Element>) -> Self::AddOutput {
    WidgetBuilder(self.add(into_element.into()))
  }
}

impl<E: El> StateReduce for HeapList<E> where {
  type ReduceOutput = WidgetBuilder<Self>;
  fn reduce(self, reduce_fn: impl FnOnce(Vec<E>) -> E) -> Self::ReduceOutput {
    let (vec, reserve_additional) = self.unwrap();
    let element = reduce_fn(vec);
    WidgetBuilder(HeapList::One(element, reserve_additional))
  }
}

impl<E: El> StateMap for HeapList<E> {
  type MapOutput = WidgetBuilder<Self>;
  #[inline]
  fn map_last(self, map_fn: impl FnOnce(E) -> E) -> Self::MapOutput {
    let mapped = match self {
      HeapList::Zero => panic!("builder should have at least 1 element"),
      HeapList::One(element, reserve_additional) => HeapList::One(map_fn(element), reserve_additional),
      HeapList::Many(mut vec) => {
        let element = vec.pop()
          .unwrap_or_else(|| panic!("builder should have at least 1 element"));
        let element = map_fn(element);
        vec.push(element);
        HeapList::Many(vec)
      }
    };
    WidgetBuilder(mapped)
  }
}

impl<E: El> StateTakeAll for HeapList<E> where {
  #[inline]
  fn take_all(self) -> Vec<E> {
    self.unwrap().0
  }
}

impl<E: El> StateTake for HeapList<E> {
  #[inline]
  fn take(self) -> E {
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
