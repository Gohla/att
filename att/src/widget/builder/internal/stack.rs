//! Stack implementation: full compile-time safety and zero-cost, but every operation changes the type of the state.
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

use crate::widget::builder::WidgetBuilder;

use super::{AnyState, ManyState, OneState, Types};

/// Algebraic stack list constructor.
pub struct Cons<E, Rest>(E, Rest);
/// Empty list.
#[repr(transparent)]
pub struct Nil<E>(PhantomData<E>);
impl<E> Default for Nil<E> {
  #[inline]
  fn default() -> Self { Self(PhantomData::default()) }
}

/// Internal trait for algebraic stack list operations.
pub trait StackList: Sized {
  /// Type of elements in the list.
  type E;
  /// The length of this list.
  const LEN: usize;
  /// Return a new list with `element` added to it.
  #[inline]
  fn add(self, element: Self::E) -> Cons<Self::E, Self> {
    Cons(element, self)
  }
  /// Consume the elements from this list into a [`Vec`].
  #[inline]
  fn consume(self) -> Vec<Self::E> {
    let mut vec = Vec::with_capacity(Self::LEN);
    self.add_to_vec(&mut vec);
    vec
  }
  /// Add the elements of this list into `vec`.
  fn add_to_vec(self, vec: &mut Vec<Self::E>);
}

// Implement `StackList` for `Cons` and `Nil`.
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

// Implement internal traits for all types implementing `StackList`.
impl<'a, M, R, L> Types<'a> for L where
  M: 'a,
  R: Renderer + 'a,
  L: StackList<E=Element<'a, M, R>>
{
  type Message = M;
  type Renderer = R;
  type Theme = R::Theme;
}
impl<'a, M, R, L> AnyState<'a> for L where
  M: 'a,
  R: Renderer + 'a,
  L: StackList<E=Element<'a, M, R>>
{
  type AddOutput = WidgetBuilder<Cons<Element<'a, M, R>, Self>>;
  #[inline]
  fn add(self, element: Element<'a, M, R>) -> Self::AddOutput {
    WidgetBuilder(StackList::add(self, element))
  }

  type ConsumeOutput = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
  #[inline]
  fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeOutput {
    let vec = self.consume();
    let element = produce(vec);
    WidgetBuilder(Cons(element, Nil::default()))
  }

  #[inline]
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    self.consume()
  }
}
impl<'a, M, R, L> ManyState<'a> for Cons<Element<'a, M, R>, L> where
  M: 'a,
  R: Renderer + 'a,
  L: StackList<E=Element<'a, M, R>>
{
  type MapOutput = WidgetBuilder<Cons<Element<'a, M, R>, L>>;
  #[inline]
  fn map_last<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapOutput {
    let Cons(element, rest) = self;
    let element = map(element);
    WidgetBuilder(Cons(element, rest))
  }
}
impl<'a, M, R> OneState<'a> for Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>> where
  M: 'a,
  R: Renderer + 'a
{
  #[inline]
  fn take(self) -> Element<'a, M, R> {
    self.0
  }
}
