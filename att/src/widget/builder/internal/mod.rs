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

/// Internal trait for widget builder state of any length, providing add, consume, and take_vec operations.
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

  /// Take the [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_vec(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
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
