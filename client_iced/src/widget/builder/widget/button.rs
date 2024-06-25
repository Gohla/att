use iced::Element;
use iced::widget::Button;
use iced::widget::button;

use crate::widget::builder::state::{Elem, ElemM};

use super::super::state::State;

pub trait ButtonActions {
  type Change<F>;
  fn on_press<F>(self, on_press: F) -> Self::Change<F>;
}

type Btn<'a, S, M> = Button<'a, M, <S as State>::Theme, <S as State>::Renderer>;

pub trait CreateButton<'a, S> where
  S: State,
  S::Theme: button::Catalog
{
  type Message: Clone;
  fn create(
    self,
    content: impl Into<ElemM<'a, S, Self::Message>>,
    modify: impl FnOnce(Btn<'a, S, Self::Message>) -> Btn<'a, S, Self::Message>,
  ) -> Elem<'a, S>;
}

/// Passthrough which does not modify the message type, thus the message type must implement [`Clone`].
pub struct ButtonPassthrough;

impl ButtonActions for ButtonPassthrough {
  type Change<F> = ButtonFunctions<F>;
  #[inline]
  fn on_press<F>(self, on_press: F) -> Self::Change<F> { ButtonFunctions { on_press } }
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
  type Change<F> = ButtonFunctions<F>;
  #[inline]
  fn on_press<F>(self, on_press: F) -> Self::Change<F> { ButtonFunctions { on_press } }
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
    let button = modify(Button::new(content));
    let button = button.on_press(());
    Element::new(button).map(move |_| (self.on_press)())
  }
}
