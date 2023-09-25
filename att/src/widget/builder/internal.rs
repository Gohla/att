use std::marker::PhantomData;

use iced::advanced::Renderer;
use iced::Element;
use iced::widget::TextInput;

use super::{TextInputStyleSheet, TextRenderer};
use super::WidgetBuilder;

/// Internal trait for access to element types.
pub trait Types<'a> {
  /// [`Element`] message type.
  type Message: 'a;
  /// [`Element`] renderer type.
  type Renderer: Renderer<Theme=Self::Theme> + 'a;
  /// Theme type of the [`Self::Renderer`].
  type Theme;
}

/// Internal trait for widget builder state of any length, providing add and consume operations.
pub trait AnyState<'a>: Types<'a> {
  /// Builder type to return from [`Self::add`].
  type AddBuilder;
  /// Add `element` onto `self`, then return a [new builder](Self::AddBuilder) with those elements.
  fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddBuilder;

  /// Builder type to return from [`Self::consume`].
  type ConsumeBuilder;
  /// Consume the [elements](Element) from `self` into a [`Vec`], call `produce` on that [`Vec`] to create a new
  /// [`Element`], then return a [new builder](Self::ConsumeBuilder) with that element.
  fn consume<F>(self, produce: F) -> Self::ConsumeBuilder where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>;
}

/// Internal trait for widget builder state of length 1, providing map and take operations.
pub trait OneState<'a>: Types<'a> {
  /// Builder type to return from [`Self::map`].
  type MapBuilder;
  /// Take the single [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapBuilder) with that element.
  fn map<F>(self, map: F) -> Self::MapBuilder where
    F: FnOnce(Element<'a, Self::Message, Self::Renderer>) -> Element<'a, Self::Message, Self::Renderer>;

  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Element<'a, Self::Message, Self::Renderer>;
}


// Stack implementation: full compile-time safety and zero-cost, but every operation changes the type of the state.
// Inspirations:
// - https://github.com/lloydmeta/frunk/blob/master/core/src/hlist.rs
//   - https://beachape.com/blog/2017/03/12/gentle-intro-to-type-level-recursion-in-Rust-from-zero-to-frunk-hlist-sculpting/
// - https://github.com/grego/slist/blob/master/src/lib.rs
// - https://rust-unofficial.github.io/too-many-lists/infinity-stack-allocated.html
// - https://willcrichton.net/notes/type-level-programming/
//   - https://willcrichton.net/notes/gats-are-hofs/
//   - https://github.com/willcrichton/tyrade

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
  type AddBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Self>>;
  #[inline]
  fn add(self, element: Element<'a, M, R>) -> Self::AddBuilder {
    WidgetBuilder(StackList::add(self, element))
  }

  type ConsumeBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
  #[inline]
  fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeBuilder {
    let vec = self.consume();
    let element = produce(vec);
    WidgetBuilder(Cons(element, Nil::default()))
  }
}

impl<'a, M, R> OneState<'a> for Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>> where
  M: 'a,
  R: Renderer + 'a
{
  type MapBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
  #[inline]
  fn map<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapBuilder {
    let element = self.take();
    let element = map(element);
    WidgetBuilder(Cons(element, Nil::default()))
  }

  #[inline]
  fn take(self) -> Element<'a, M, R> {
    self.0
  }
}


// Heap implementation: run-time type safety, not zero-cost, but type does not change.

pub enum Heap<E> {
  Any(Vec<E>),
  One(E, usize),
}
impl<E> Heap<E> {
  #[inline]
  pub fn new() -> Self { Self::Any(Vec::new()) }
  #[inline]
  pub fn with_capacity(capacity: usize) -> Self { Self::Any(Vec::with_capacity(capacity)) }

  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    match self {
      Heap::Any(ref mut vec) => vec.reserve(additional),
      Heap::One(_, reserve_additional) => *reserve_additional += additional,
    }
  }

  #[inline]
  fn push(self, new_element: E) -> Self {
    match self {
      Heap::Any(mut vec) => {
        vec.push(new_element);
        Heap::Any(vec)
      },
      Heap::One(element, reserve_additional) => {
        let vec = if reserve_additional > 0 {
          let mut vec = Vec::with_capacity(2 + reserve_additional);
          vec.push(element);
          vec.push(new_element);
          vec
        } else {
          vec![element, new_element]
        };
        Heap::Any(vec)
      },
    }
  }
  #[inline]
  fn consume(self) -> Vec<E> {
    match self {
      Heap::Any(vec) => vec,
      Heap::One(element, _) => vec![element], // Note: ignore reserve_additional, since the vec will be consumed as-is.
    }
  }
  #[inline]
  fn take(self) -> (E, usize) {
    match self {
      Heap::Any(mut vec) => {
        let len = vec.len();
        let 1 = len else {
          panic!("builder should have precisely 1 element, but it has {}", len);
        };
        let element = vec.drain(..).next().unwrap();
        (element, 0)
      }
      Heap::One(element, reserve_additional) => (element, reserve_additional),
    }
  }
}

impl<'a, M, R> Types<'a> for Heap<Element<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a,
{
  type Message = M;
  type Renderer = R;
  type Theme = R::Theme;
}

impl<'a, M, R> AnyState<'a> for Heap<Element<'a, M, R>> where
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
    let vec = self.consume();
    let element = produce(vec);
    WidgetBuilder(Heap::One(element, 0))
  }
}

impl<'a, M, R> OneState<'a> for Heap<Element<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a
{
  type MapBuilder = WidgetBuilder<Self>;
  #[inline]
  fn map<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapBuilder {
    let (element, reserve_additional) = self.take();
    let element = map(element);
    WidgetBuilder(Heap::One(element, reserve_additional))
  }

  #[inline]
  fn take(self) -> Element<'a, M, R> {
    self.take().0
  }
}


// Text input internals

pub trait TextInputActions<'a, M> {
  type Change;
  fn on_input<F: Fn(String) -> M + 'a>(self, on_input: F) -> Self::Change;
  fn on_paste<F: Fn(String) -> M + 'a>(self, on_paste: F) -> Self::Change;
  fn on_submit<F: Fn() -> M + 'a>(self, on_submit: F) -> Self::Change;
}
pub trait CreateTextInput<'a, S: Types<'a>> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet
{
  type Message: Clone;
  fn create<F>(self, placeholder: &str, value: &str, modify: F) -> Element<'a, S::Message, S::Renderer> where
    F: FnOnce(TextInput<'a, Self::Message, S::Renderer>) -> TextInput<'a, Self::Message, S::Renderer>;
}

pub struct TextInputPassthrough;
impl<'a, M> TextInputActions<'a, M> for TextInputPassthrough {
  type Change = TextInputFunctions<'a, M>;
  #[inline]
  fn on_input<F: Fn(String) -> M + 'a>(self, on_input: F) -> Self::Change {
    TextInputFunctions { on_input: Some(Box::new(on_input)), ..Default::default() }
  }
  #[inline]
  fn on_paste<F: Fn(String) -> M + 'a>(self, on_paste: F) -> Self::Change {
    TextInputFunctions { on_paste: Some(Box::new(on_paste)), ..Default::default() }
  }
  #[inline]
  fn on_submit<F: Fn() -> M + 'a>(self, on_submit: F) -> Self::Change {
    TextInputFunctions { on_submit: Some(Box::new(on_submit)), ..Default::default() }
  }
}
impl<'a, S: Types<'a>> CreateTextInput<'a, S> for TextInputPassthrough where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
  S::Message: Clone,
{
  type Message = S::Message;
  #[inline]
  fn create<F>(self, placeholder: &str, value: &str, modify: F) -> Element<'a, S::Message, S::Renderer> where
    F: FnOnce(TextInput<'a, Self::Message, S::Renderer>) -> TextInput<'a, Self::Message, S::Renderer>
  {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    Element::new(text_input)
  }
}

pub struct TextInputFunctions<'a, M> {
  // TODO: don't use boxed functions here, since iced will box them again?
  on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
  on_paste: Option<Box<dyn Fn(String) -> M + 'a>>,
  on_submit: Option<Box<dyn Fn() -> M + 'a>>,
}
impl<'a, M> Default for TextInputFunctions<'a, M> {
  fn default() -> Self { Self { on_input: None, on_paste: None, on_submit: None } }
}
impl<'a, M> TextInputActions<'a, M> for TextInputFunctions<'a, M> {
  type Change = Self;
  #[inline]
  fn on_input<F: Fn(String) -> M + 'a>(mut self, on_input: F) -> Self::Change {
    self.on_input = Some(Box::new(on_input));
    self
  }
  #[inline]
  fn on_paste<F: Fn(String) -> M + 'a>(mut self, on_paste: F) -> Self::Change {
    self.on_paste = Some(Box::new(on_paste));
    self
  }
  #[inline]
  fn on_submit<F: Fn() -> M + 'a>(mut self, on_submit: F) -> Self::Change {
    self.on_submit = Some(Box::new(on_submit));
    self
  }
}
impl<'a, S: Types<'a>> CreateTextInput<'a, S> for TextInputFunctions<'a, S::Message> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
{
  type Message = TextInputAction;
  #[inline]
  fn create<F>(self, placeholder: &str, value: &str, modify: F) -> Element<'a, S::Message, S::Renderer> where
    F: FnOnce(TextInput<'a, Self::Message, S::Renderer>) -> TextInput<'a, Self::Message, S::Renderer>
  {
    let mut text_input = TextInput::new(placeholder, value);
    text_input = modify(text_input);
    if self.on_input.is_some() {
      text_input = text_input.on_input(TextInputAction::Input);
    }
    if self.on_paste.is_some() {
      text_input = text_input.on_paste(TextInputAction::Paste);
    }
    if self.on_submit.is_some() {
      text_input = text_input.on_submit(TextInputAction::Submit);
    }
    Element::new(text_input)
      .map(move |m| match m {
        TextInputAction::Input(input) => (self.on_input.as_ref().unwrap())(input),
        TextInputAction::Paste(input) => (self.on_paste.as_ref().unwrap())(input),
        TextInputAction::Submit => (self.on_submit.as_ref().unwrap())(),
      })
  }
}
#[derive(Clone)]
pub enum TextInputAction {
  Input(String),
  Paste(String),
  Submit,
}
