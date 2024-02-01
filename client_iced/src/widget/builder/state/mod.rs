use iced::advanced::{Renderer, Widget};
use iced::Element;
use iced::widget::Space;

pub mod stack;
pub mod heap;
pub mod once;

pub trait Elem {
  /// [`Element`] message type.
  type Message;
  /// [`Element`] theme type.
  type Theme;
  /// [`Element`] renderer type.
  type Renderer: Renderer;
}
impl<'a, M, T, R: Renderer> Elem for Element<'a, M, T, R> {
  type Message = M;
  type Theme = T;
  type Renderer = R;
}

/// Internal trait for widget builder state.
pub trait State {
  /// Type of [elements](Element) contained in this state.
  type Element: Elem;

  /// [`Element`] message type.
  type Message;
  /// [`Element`] theme type.
  type Theme;
  /// [`Element`] renderer type.
  type Renderer: Renderer;
}

/// Internal trait for adding to widget builder state.
pub trait StateAdd: State {
  /// Type to return from [`Self::add`].
  type AddOutput;
  /// Add `element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn add<I: Into<Self::Element>>(self, into_elem: I) -> Self::AddOutput;
}

/// Internal trait for consuming widget builder state.
pub trait StateConsume<'a>: State {
  /// Type to return from [`Self::consume`].
  type ConsumeOutput;
  /// Consume all [elements](Element) from `self` into a [`Vec`], call `f` on that [`Vec`] to create a new [`Element`],
  /// then return a [new builder](Self::ConsumeOutput) with that element.
  fn consume<F>(self, f: F) -> Self::ConsumeOutput where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}

/// Internal trait for mapping widget builder state.
pub trait StateMap<'a>: State {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last<F>(self, map: F) -> Self::MapOutput where
    F: FnOnce(Element<'a, Self::Message, Self::Theme, Self::Renderer>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
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
