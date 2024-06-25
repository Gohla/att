//! Stack-allocated list:
//!
//! - Full compile-time safety.
//! - Low run-time overhead. TODO: benchmark this
//! - Some compile-time overhead. TODO: benchmark this
//! - Every operation changes the type of the list.
//!
//! Inspirations:
//!
//! - https://github.com/lloydmeta/frunk/blob/master/core/src/hlist.rs
//!   - https://beachape.com/blog/2017/03/12/gentle-intro-to-type-level-recursion-in-Rust-from-zero-to-frunk-hlist-sculpting/
//! - https://github.com/grego/slist/blob/master/src/lib.rs
//! - https://rust-unofficial.github.io/too-many-lists/infinity-stack-allocated.html
//! - https://willcrichton.net/notes/type-level-programming/
//!   - https://willcrichton.net/notes/gats-are-hofs/
//!   - https://github.com/willcrichton/tyrade

use std::marker::PhantomData;

use crate::WidgetBuilder;

use super::{El, State, StateAppend, StateMap, StateReduce, StateTake, StateTakeAll};

/// List constructor.
pub struct Cons<E, Rest>(E, Rest);

impl<E, Rest> Cons<E, Rest> {
  #[inline]
  fn map(mut self, map_fn: impl FnOnce(E) -> E) -> Self {
    self.0 = map_fn(self.0);
    self
  }
}


/// Empty list.
#[repr(transparent)]
pub struct Nil<E>(PhantomData<E>);

impl<E> Default for Nil<E> {
  #[inline]
  fn default() -> Self { Self(PhantomData::default()) }
}


/// List trait.
trait List: Sized {
  /// Type of elements in the list.
  type E;

  /// The length of this list.
  const LEN: usize;

  /// Create a new list with `element`.
  #[inline]
  fn one(element: Self::E) -> Cons<Self::E, Nil<Self::E>> { Cons(element, Nil::default()) }

  /// Return a new list with `element` appended to it.
  #[inline]
  fn append(self, element: Self::E) -> Cons<Self::E, Self> { Cons(element, self) }

  /// Return a new list with the element reduced from `reduce_fn`.
  #[inline]
  fn reduce(self, reduce_fn: impl FnOnce(Vec<Self::E>) -> Self::E) -> Cons<Self::E, Nil<Self::E>> {
    Self::one(reduce_fn(self.to_vec()))
  }

  /// Collect the elements from this list into a [`Vec`].
  #[inline]
  fn to_vec(self) -> Vec<Self::E> {
    let mut vec = Vec::with_capacity(Self::LEN);
    self.add_to_vec(&mut vec);
    vec
  }

  /// Add the elements of this list into `vec`.
  fn add_to_vec(self, vec: &mut Vec<Self::E>);
}

impl<E, Rest: List<E=E>> List for Cons<E, Rest> {
  type E = E;

  const LEN: usize = 1 + Rest::LEN;

  #[inline]
  fn add_to_vec(self, vec: &mut Vec<Self::E>) {
    // Note: visiting in reverse order to get Vec that is correctly ordered w.r.t. `append`.
    self.1.add_to_vec(vec);
    vec.push(self.0);
  }
}

impl<E> List for Nil<E> {
  type E = E;

  const LEN: usize = 0;

  #[inline]
  fn add_to_vec(self, _vec: &mut Vec<E>) {}
}


// Implement state traits for all types implementing `StackList`.

impl<E: El, L: List<E=E>> State for L {
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E: El, L: List<E=E>> StateAppend for L {
  type AddOutput = WidgetBuilder<Cons<E, Self>>;
  #[inline]
  fn append(self, into_element: impl Into<E>) -> Self::AddOutput { WidgetBuilder(self.append(into_element.into())) }
}

impl<E: El, L: List<E=E>> StateReduce for L {
  type ReduceOutput = WidgetBuilder<Cons<E, Nil<E>>>;
  #[inline]
  fn reduce(self, reduce_fn: impl FnOnce(Vec<E>) -> E) -> Self::ReduceOutput { WidgetBuilder(self.reduce(reduce_fn)) }
}

impl<E: El, L: List<E=E>> StateMap for Cons<E, L> {
  type MapOutput = WidgetBuilder<Cons<E, L>>;
  #[inline]
  fn map_last(self, map_fn: impl FnOnce(E) -> E) -> Self::MapOutput { WidgetBuilder(self.map(map_fn)) }
}

impl<E: El, L: List<E=E>> StateTakeAll for L {
  #[inline]
  fn take_all(self) -> Vec<E> { self.to_vec() }
}

impl<E: El> StateTake for Cons<E, Nil<E>> {
  #[inline]
  fn take(self) -> E { self.0 }
}
