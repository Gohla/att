use iced::Element;
use iced::widget::Button;
use iced::widget::button::StyleSheet as ButtonStyleSheet;

use super::super::state::State;

pub trait ButtonActions<'a, M> {
  type Change;
  fn on_press(self, on_press: impl Fn() -> M + 'a) -> Self::Change;
}

type B<'a, M, S> = Button<'a, M, <S as State>::Theme, <S as State>::Renderer>;

pub trait CreateButton<'a, S> where
  S: State,
  S::Theme: ButtonStyleSheet
{
  type Message: Clone;
  fn create(
    self,
    content: impl Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    modify: impl FnOnce(B<'a, Self::Message, S>) -> B<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer>;
}

/// Passthrough which does not modify the message type, thus the message type must implement [`Clone`].
pub struct ButtonPassthrough;
impl<'a, M> ButtonActions<'a, M> for ButtonPassthrough {
  type Change = ButtonFunctions<'a, M>;

  #[inline]
  fn on_press(self, on_press: impl Fn() -> M + 'a) -> Self::Change {
    ButtonFunctions { on_press: Box::new(on_press) }
  }
}
impl<'a, S> CreateButton<'a, S> for ButtonPassthrough where
  S: State + 'a,
  S::Message: Clone,
  S::Theme: ButtonStyleSheet,
{
  type Message = S::Message;

  #[inline]
  fn create(
    self,
    content: impl Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    modify: impl FnOnce(B<'a, Self::Message, S>) -> B<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer> {
    let mut button = Button::new(content);
    button = modify(button);
    Element::new(button)
  }
}

/// Modify message type to `()` which is [`Clone`], without our callback needing to implement clone.
pub struct ButtonFunctions<'a, M> {
  on_press: Box<dyn Fn() -> M + 'a>,
}
impl<'a, M> ButtonActions<'a, M> for ButtonFunctions<'a, M> {
  type Change = Self;

  #[inline]
  fn on_press(mut self, on_press: impl Fn() -> M + 'a) -> Self::Change {
    self.on_press = Box::new(on_press);
    self
  }
}
impl<'a, S> CreateButton<'a, S> for ButtonFunctions<'a, S::Message> where
  S: State + 'a,
  S::Theme: ButtonStyleSheet,
{
  type Message = ();

  #[inline]
  fn create(
    self,
    content: impl Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    modify: impl FnOnce(B<'a, Self::Message, S>) -> B<'a, Self::Message, S>,
  ) -> Element<'a, S::Message, S::Theme, S::Renderer> {
    let mut button = Button::new(content)
      .on_press(());
    button = modify(button);
    Element::new(button).map(move |_| (self.on_press)())
  }
}
