use iced::Element;
use iced::widget::Button;
use iced::widget::button;

use crate::internal::state::{Elem, ElemM};

use super::super::state::State;

/// Internal trait for type-changing [Button] actions.
pub trait ButtonActions {
  /// Type after changing [Self::on_press].
  type ChangeOnPress<F>;

  /// Change the [Button::on_press] action to `on_press`.
  fn on_press<F>(self, on_press: F) -> Self::ChangeOnPress<F>;
}

/// Internal type alias for a [Button].
type Btn<'a, S, M> = Button<'a, M, <S as State>::Theme, <S as State>::Renderer>;

/// Internal trait for creating a [Button].
pub trait CreateButton<'a, S> where
  S: State,
  S::Theme: button::Catalog
{
  /// Type of messages. Must implement [Clone] because iced requires that.
  type Message: Clone;

  /// Create a button element from `content`, then let `modify` modify the button.
  fn create(
    self,
    content: impl Into<ElemM<'a, S, Self::Message>>,
    modify: impl FnOnce(Btn<'a, S, Self::Message>) -> Btn<'a, S, Self::Message>,
  ) -> Elem<'a, S>;
}


/// Passthrough which does not modify the message type, thus the message type must implement [`Clone`].
pub struct ButtonPassthrough;

impl ButtonActions for ButtonPassthrough {
  type ChangeOnPress<F> = ButtonFunctions<F>;

  #[inline]
  fn on_press<F>(self, on_press: F) -> Self::ChangeOnPress<F> { ButtonFunctions { on_press } }
}

impl<'a, S> CreateButton<'a, S> for ButtonPassthrough where
  S: State + 'a,
  S::Message: Clone,
  S::Theme: button::Catalog,
{
  type Message = S::Message;

  #[inline]
  fn create(
    self,
    content: impl Into<ElemM<'a, S, Self::Message>>,
    modify: impl FnOnce(Btn<'a, S, Self::Message>) -> Btn<'a, S, Self::Message>,
  ) -> Elem<'a, S> {
    Element::new(modify(Button::new(content)))
  }
}


/// Modify message type to `()` which is [`Clone`], without our callback needing to implement clone.
pub struct ButtonFunctions<FP> {
  on_press: FP,
}

impl<FP> ButtonActions for ButtonFunctions<FP> {
  type ChangeOnPress<F> = ButtonFunctions<F>;

  #[inline]
  fn on_press<F>(self, on_press: F) -> Self::ChangeOnPress<F> { ButtonFunctions { on_press } }
}

impl<'a, S, FP> CreateButton<'a, S> for ButtonFunctions<FP> where
  S: State + 'a,
  S::Theme: button::Catalog,
  FP: Fn() -> S::Message + 'a,
{
  type Message = ();

  #[inline]
  fn create(
    self,
    content: impl Into<ElemM<'a, S, Self::Message>>,
    modify: impl FnOnce(Btn<'a, S, Self::Message>) -> Btn<'a, S, Self::Message>,
  ) -> Elem<'a, S> {
    let button = Button::new(content);
    let button = modify(button);
    let button = button.on_press(());
    Element::new(button).map(move |_| (self.on_press)())
  }
}
