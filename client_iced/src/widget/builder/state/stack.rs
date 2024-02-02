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

use super::{El, State, StateAdd, StateConsume, StateMap, StateTake, StateTakeAll};
use super::super::WidgetBuilder;

/// Algebraic stack list constructor.
pub struct Cons<E, Rest>(E, Rest);
/// Empty list.
#[repr(transparent)]
pub struct Nil<E>(PhantomData<E>);
impl<E> Default for Nil<E> {
  #[inline]
  fn default() -> Self { Self(PhantomData::default()) }
}

/// Stack-allocated list.
trait StackList: Sized {
  /// Type of elements in the list.
  type E;
  /// The length of this list.
  const LEN: usize;
  /// Return a new list with `element` added to it.
  #[inline]
  fn add(self, element: Self::E) -> Cons<Self::E, Self> {
    Cons(element, self)
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

impl<E, Rest: StackList<E=E>> StackList for Cons<E, Rest> {
  type E = E;
  const LEN: usize = 1 + Rest::LEN;
  #[inline]
  fn add_to_vec(self, vec: &mut Vec<Self::E>) {
    // Note: visiting in reverse order to get Vec that is correctly ordered w.r.t. `add`.
    self.1.add_to_vec(vec);
    vec.push(self.0);
  }
}
impl<E> StackList for Nil<E> {
  type E = E;
  const LEN: usize = 0;
  #[inline]
  fn add_to_vec(self, _vec: &mut Vec<E>) {}
}


// Implement state traits for all types implementing `StackList`.

impl<E: El, L: StackList<E=E>> State for L {
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E: El, L: StackList<E=E>> StateAdd for L {
  type AddOutput = WidgetBuilder<Cons<E, Self>>;
  #[inline]
  fn add(self, into: impl Into<Self::Element>) -> Self::AddOutput {
    WidgetBuilder(StackList::add(self, into.into()))
  }
}

impl<E: El, L: StackList<E=E>> StateConsume for L {
  type ConsumeOutput = WidgetBuilder<Cons<E, Nil<E>>>;
  #[inline]
  fn consume(self, produce: impl FnOnce(Vec<E>) -> E) -> Self::ConsumeOutput {
    let vec = self.to_vec();
    let element = produce(vec);
    WidgetBuilder(Cons(element, Nil::default()))
  }
}

impl<E: El, L: StackList<E=E>> StateMap for Cons<E, L> {
  type MapOutput = WidgetBuilder<Cons<E, L>>;
  #[inline]
  fn map_last(self, map: impl FnOnce(E) -> E) -> Self::MapOutput {
    let Cons(element, rest) = self;
    let element = map(element);
    WidgetBuilder(Cons(element, rest))
  }
}

impl<E: El, L: StackList<E=E>> StateTakeAll for L {
  #[inline]
  fn take_all(self) -> Vec<E> {
    self.to_vec()
  }
}

impl<E: El> StateTake for Cons<E, Nil<E>> {
  #[inline]
  fn take(self) -> E {
    self.0
  }
}
