use iced::advanced::Renderer;
use iced::Element;

pub mod stack;
pub mod heap;
pub mod once;

/// Internal trait for element types.
pub trait El {
  /// [`Element`] message type.
  type Message;
  /// [`Element`] theme type.
  type Theme;
  /// [`Element`] renderer type.
  type Renderer: Renderer;
}
impl<'a, M, T, R: Renderer> El for Element<'a, M, T, R> {
  type Message = M;
  type Theme = T;
  type Renderer = R;
}

/// Internal trait for widget builder state.
pub trait State {
  /// Type of [elements](Element) contained in this state.
  type Element: El<Message=Self::Message, Theme=Self::Theme, Renderer=Self::Renderer>;

  /// [`Element`] message type.
  type Message;
  /// [`Element`] theme type.
  type Theme;
  /// [`Element`] renderer type.
  type Renderer: Renderer;
}

/// Internal type alias for [elements](Element) with lifetime `'a`, message type `M`, theme type `S::Theme`, and
/// renderer type `S::Renderer`.
pub type ElemM<'a, S, M> = Element<'a, M, <S as State>::Theme, <S as State>::Renderer>;
/// Internal type alias for [elements](Element) with lifetime `'a`, message type `S::Message`, theme type `S::Theme`,
/// and renderer type `S::Renderer`.
pub type Elem<'a, S> = ElemM<'a, S, <S as State>::Message>;

/// Internal trait for adding to widget builder state.
pub trait StateAppend: State {
  /// Type to return from [`Self::append`].
  type AddOutput;
  /// Append `into_element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn append(self, into_element: impl Into<Self::Element>) -> Self::AddOutput;
}

/// Internal trait for reducing widget builder state.
pub trait StateReduce: State {
  /// Type to return from [`Self::reduce`].
  type ReduceOutput;
  /// Collect all [elements](Element) from `self` into a [`Vec`], call `reduce` on that [`Vec`] to create a new
  /// [`Element`], then return a [new builder](Self::ReduceOutput) with that element.
  fn reduce(self, reduce_fn: impl FnOnce(Vec<Self::Element>) -> Self::Element) -> Self::ReduceOutput;
}

/// Internal trait for mapping widget builder state.
pub trait StateMap: State {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last(self, map_fn: impl FnOnce(Self::Element) -> Self::Element) -> Self::MapOutput;
}

/// Internal trait taking all widget builder state.
pub trait StateTakeAll: State {
  /// Take all [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_all(self) -> Vec<Self::Element>;
}

/// Internal trait taking single widget builder state.
pub trait StateTake: State {
  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Self::Element;
}
