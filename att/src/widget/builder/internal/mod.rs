use iced::advanced::Renderer;
use iced::Element;

pub mod stack;
pub mod heap;
pub mod text_input;
pub mod button;

/// Internal trait for access to element types.
pub trait Types<'a> {
  /// [`Element`] message type.
  type Message: 'a;
  /// [`Element`] renderer type.
  type Renderer: Renderer<Theme=Self::Theme> + 'a;
  /// Theme type of the [`Self::Renderer`].
  type Theme;
}

/// Internal trait for widget builder state of any length, providing `add`, `consume`, and `take_all` operations.
pub trait AnyState<'a>: Types<'a> {
  /// Type to return from [`Self::add`].
  type AddOutput;
  /// Add `element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddOutput;

  /// Type to return from [`Self::consume`].
  type ConsumeOutput;
  /// Consume all [elements](Element) from `self` into a [`Vec`], call `f` on that [`Vec`] to create a new [`Element`],
  /// then return a [new builder](Self::ConsumeOutput) with that element.
  fn consume<F>(self, f: F) -> Self::ConsumeOutput where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>;

  /// Take all [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
}

/// Internal trait for widget builder state of length 1 or more, providing the `map` operatio.
pub trait ManyState<'a>: Types<'a> {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last<F>(self, map: F) -> Self::MapOutput where
    F: FnOnce(Element<'a, Self::Message, Self::Renderer>) -> Element<'a, Self::Message, Self::Renderer>;
}

/// Internal trait for widget builder state of length 1, providing the `take_one` operation.
pub trait OneState<'a>: Types<'a> {
  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Element<'a, Self::Message, Self::Renderer>;
}
