use iced::advanced::Renderer;
use iced::Element;

pub mod stack;
pub mod heap;
pub mod once;

/// Internal trait for access to element types.
pub trait StateTypes<'a> {
  /// [`Element`] message type.
  type Message: 'a;
  /// [`Element`] renderer type.
  type Renderer: Renderer<Theme=Self::Theme> + 'a;
  /// Theme type of the [`Self::Renderer`].
  type Theme;
}

/// Internal trait for adding to widget builder state.
pub trait StateAdd<'a>: StateTypes<'a> {
  /// Type to return from [`Self::add`].
  type AddOutput;
  /// Add `element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddOutput;
}

/// Internal trait for consuming widget builder state.
pub trait StateConsume<'a>: StateTypes<'a> {
  /// Type to return from [`Self::consume`].
  type ConsumeOutput;
  /// Consume all [elements](Element) from `self` into a [`Vec`], call `f` on that [`Vec`] to create a new [`Element`],
  /// then return a [new builder](Self::ConsumeOutput) with that element.
  fn consume<F>(self, f: F) -> Self::ConsumeOutput where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>;
}

/// Internal trait for mapping widget builder state.
pub trait StateMap<'a>: StateTypes<'a> {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last<F>(self, map: F) -> Self::MapOutput where
    F: FnOnce(Element<'a, Self::Message, Self::Renderer>) -> Element<'a, Self::Message, Self::Renderer>;
}

/// Internal trait taking all widget builder state.
pub trait StateTakeAll<'a>: StateTypes<'a> {
  /// Take all [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
}

/// Internal trait taking single widget builder state.
pub trait StateTake<'a>: StateTypes<'a> {
  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Element<'a, Self::Message, Self::Renderer>;
}
