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
impl<'a, M: 'a, T: 'a, R: Renderer + 'a> Elem for Element<'a, M, T, R> {
  type Message = M;
  type Theme = T;
  type Renderer = R;
}

// pub trait IntoElem {
//   /// [`Element`] message type.
//   type Message;
//   /// [`Element`] theme type.
//   type Theme;
//   /// [`Element`] renderer type.
//   type Renderer: Renderer;
//
//   type Element;
//   fn into_elem(self) -> Self::Element;
// }
// impl<'a, M, T, R> IntoElem for Space {
//   type Message = M;
//   type Theme = T;
//   type Renderer = R;
//   type Element = Element<'a, M, T, R>;
//
//   fn into_elem(self) -> Self::Element {
//     self.into()
//   }
// }


// pub trait IntoElem<E: Elem> {
//   fn into_elem(self) -> E;
// }
// impl<'a, M: 'a, T: 'a, R: Renderer + 'a> IntoElem<Element<'a, M, T, R>> for Space {
//   fn into_elem(self) -> Element<'a, M, T, R> {
//     self.into()
//   }
// }
// impl<E: Elem + From<W>, W: Widget<E::Message, E::Theme, E::Renderer>> IntoElem<E> for W {
//   fn into_elem(self) -> E {
//     self.into()
//   }
// }

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
pub trait StateTypes {
  type Element: Elem;
  /// [`Element`] message type.
  type Message;
  /// [`Element`] theme type.
  type Theme;
  /// [`Element`] renderer type.
  type Renderer: Renderer;
}

/// Internal trait for adding to widget builder state.
pub trait StateAdd: StateTypes {
  //type Element: Elem;
  /// Type to return from [`Self::add`].
  type AddOutput;
  /// Add `element` onto `self`, then return a [new builder](Self::AddOutput) with those elements.
  fn add<I: Into<Self::Element>>(self, into_elem: I) -> Self::AddOutput;
}

/// Internal trait for consuming widget builder state.
pub trait StateConsume<'a>: StateTypes {
  /// Type to return from [`Self::consume`].
  type ConsumeOutput;
  /// Consume all [elements](Element) from `self` into a [`Vec`], call `f` on that [`Vec`] to create a new [`Element`],
  /// then return a [new builder](Self::ConsumeOutput) with that element.
  fn consume<F>(self, f: F) -> Self::ConsumeOutput where
    F: FnOnce(Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}

/// Internal trait for mapping widget builder state.
pub trait StateMap<'a>: StateTypes {
  /// Builder type to return from [`Self::map_last`].
  type MapOutput;
  /// Take the last [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
  /// a [new builder](Self::MapOutput) with that element.
  fn map_last<F>(self, map: F) -> Self::MapOutput where
    F: FnOnce(Element<'a, Self::Message, Self::Theme, Self::Renderer>) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}

/// Internal trait taking all widget builder state.
pub trait StateTakeAll<'a>: StateTypes {
  /// Take all [elements](Element) from `self` into a [`Vec`] and return it.
  fn take_all(self) -> Vec<Element<'a, Self::Message, Self::Theme, Self::Renderer>>;
}

/// Internal trait taking single widget builder state.
pub trait StateTake<'a>: StateTypes {
  /// Take the single [`Element`] from `self` and return it.
  fn take(self) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;
}
