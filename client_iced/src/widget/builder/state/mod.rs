use iced::advanced::Renderer;
use iced::Element;

pub mod stack;
pub mod heap;
pub mod once;

pub trait Elem<'a> {
  /// [`Element`] message type.
  type Message: 'a;
  /// [`Element`] theme type.
  type Theme: 'a;
  /// [`Element`] renderer type.
  type Renderer: Renderer + 'a;
}
impl<'a, M: 'a, T: 'a, R: Renderer + 'a> Elem<'a> for Element<'a, M, T, R> {
  type Message = M;
  type Theme = T;
  type Renderer = R;
}

// pub trait IntoElem<'a> {
//   type Element: Elem<'a>;
//   fn into_elem(self) -> Self::Element;
// }
// impl<'a, M: 'a, T:'a, R: Renderer + 'a> IntoElem<'a> for Element<'a, M, T, R> {
//   type Element = Element<'a, M, T, R>;
//   fn into_elem(self) -> Self::Element {
//     self
//   }
// }
// impl<'a, M, T, R, W: Widget<M, T, R>> Into<Element<'a, M, T, R>> for  W {
//   fn into(self) -> Element<'a, M, T, R> {
//     Element::new(self)
//   }
// }


/// Internal trait for access to element types.
pub trait StateTypes<'a> {
  /// [`Element`] message type.
  type Message: 'a;
  /// [`Element`] theme type.
  type Theme: 'a;
  /// [`Element`] renderer type.
  type Renderer: Renderer + 'a;
}

/// Internal trait for adding to widget builder state.
pub trait StateAdd<'a>: StateTypes<'a> {
  type Element: Elem<'a>;
  /// Type to return from [`Self::add`].
  type AddOutput;
  /// Add `element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn add(self, element: Self::Element) -> Self::AddOutput;
}

/// Internal trait for consuming widget builder state.
pub trait StateConsume<'a>: StateTypes<'a> {
  /// Type to return from [`Self::consume`].
  type ConsumeOutput;
  /// Consume all [elements](Element) from `self` into a [`Vec`], call `f` on that [`Vec`] to create a new [`Element`],
  /// then return a [new builder](Self::ConsumeOutput) with that element.
  fn consume<F>(self, f: F) -> Self::ConsumeOutput where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}

/// Internal trait for mapping widget builder state.
pub trait StateMap<'a>: StateTypes<'a> {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last<F>(self, map: F) -> Self::MapOutput where
    F: FnOnce(Element<'a, Self::Message, Self::Theme, Self::Renderer>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}

/// Internal trait taking all widget builder state.
pub trait StateTakeAll<'a>: StateTypes<'a> {
  /// Take all [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>>;
}

/// Internal trait taking single widget builder state.
pub trait StateTake<'a>: StateTypes<'a> {
  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}
