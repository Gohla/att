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

use iced::advanced::Renderer;
use iced::Element;

use super::{Elem, StateAdd, StateConsume, StateMap, StateTake, StateTakeAll, StateTypes};
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

impl<E, L> StateTypes for L where
  E: Elem,
  L: StackList<E=E>
{
  type Element = E;
  type Message = E::Message;
  type Theme = E::Theme;
  type Renderer = E::Renderer;
}

impl<E, L> StateAdd for L where
  E: Elem,
  L: StackList<E=E>
{
  type AddOutput = WidgetBuilder<Cons<E, Self>>;
  #[inline]
  fn add<I: Into<E>>(self, into_elem: I) -> Self::AddOutput {
    WidgetBuilder(StackList::add(self, into_elem.into()))
  }
}

impl<'a, M, T, R, L> StateConsume<'a> for L where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
  L: StackList<E=Element<'a, M, T, R>>
{
  type ConsumeOutput = WidgetBuilder<Cons<Element<'a, M, T, R>, Nil<Element<'a, M, T, R>>>>;
  #[inline]
  fn consume<F: FnOnce(Vec<Element<'a, M, T, R>>) -> Element<'a, M, T, R>>(self, produce: F) -> Self::ConsumeOutput {
    let vec = self.to_vec();
    let element = produce(vec);
    WidgetBuilder(Cons(element, Nil::default()))
  }
}

impl<'a, M, T, R, L> StateMap<'a> for Cons<Element<'a, M, T, R>, L> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
  L: StackList<E=Element<'a, M, T, R>>
{
  type MapOutput = WidgetBuilder<Cons<Element<'a, M, T, R>, L>>;
  #[inline]
  fn map_last<F: FnOnce(Element<'a, M, T, R>) -> Element<'a, M, T, R>>(self, map: F) -> Self::MapOutput {
    let Cons(element, rest) = self;
    let element = map(element);
    WidgetBuilder(Cons(element, rest))
  }
}

impl<'a, M, T, R, L> StateTakeAll<'a> for L where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
  L: StackList<E=Element<'a, M, T, R>>,
{
  #[inline]
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>> {
    self.to_vec()
  }
}

impl<'a, M, T, R> StateTake<'a> for Cons<Element<'a, M, T, R>, Nil<Element<'a, M, T, R>>> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
{
  #[inline]
  fn take(self) -> Element<'a, M, T, R> {
    self.0
  }
}
