use iced::Element;
use iced::widget::Button;
use iced::widget::button::StyleSheet as ButtonStyleSheet;

use super::super::state::StateTypes;

pub trait ButtonActions<'a, M> {
  type Change;
  fn on_press<F: Fn() -> M + 'a>(self, on_press: F) -> Self::Change;
}
pub trait CreateButton<'a, S: StateTypes<'a>> where
  S::Theme: ButtonStyleSheet
{
  type Message: Clone;
  fn create<E, F>(self, content: E, modify: F) -> Element<'a, S::Message, S::Theme, S::Renderer> where
    E: Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    F: FnOnce(Button<'a, Self::Message, S::Theme, S::Renderer>) -> Button<'a, Self::Message, S::Theme, S::Renderer>;
}

pub struct ButtonPassthrough;
impl<'a, M> ButtonActions<'a, M> for ButtonPassthrough {
  type Change = ButtonFunctions<'a, M>;
  #[inline]
  fn on_press<F: Fn() -> M + 'a>(self, on_press: F) -> Self::Change {
    ButtonFunctions { on_press: Box::new(on_press) }
  }
}
impl<'a, S> CreateButton<'a, S> for ButtonPassthrough where
  S: StateTypes<'a>,
  S::Theme: ButtonStyleSheet,
  S::Message: Clone,
{
  type Message = S::Message;
  #[inline]
  fn create<E, F>(self, content: E, modify: F) -> Element<'a, S::Message, S::Theme, S::Renderer> where
    E: Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    F: FnOnce(Button<'a, Self::Message, S::Theme, S::Renderer>) -> Button<'a, Self::Message, S::Theme, S::Renderer>
  {
    let mut button = Button::new(content);
    button = modify(button);
    Element::new(button)
  }
}

pub struct ButtonFunctions<'a, M> {
  on_press: Box<dyn Fn() -> M + 'a>,
}
impl<'a, M> ButtonActions<'a, M> for ButtonFunctions<'a, M> {
  type Change = Self;
  #[inline]
  fn on_press<F: Fn() -> M + 'a>(mut self, on_press: F) -> Self::Change {
    self.on_press = Box::new(on_press);
    self
  }
}
impl<'a, S> CreateButton<'a, S> for ButtonFunctions<'a, S::Message> where
  S: StateTypes<'a>,
  S::Theme: ButtonStyleSheet,
{
  type Message = ();
  #[inline]
  fn create<E, F>(self, content: E, modify: F) -> Element<'a, S::Message, S::Theme, S::Renderer> where
    E: Into<Element<'a, Self::Message, S::Theme, S::Renderer>>,
    F: FnOnce(Button<'a, Self::Message, S::Theme, S::Renderer>) -> Button<'a, Self::Message, S::Theme, S::Renderer>
  {
    let mut button = Button::new(content)
      .on_press(());
    button = modify(button);
    Element::new(button).map(move |_| (self.on_press)())
  }
}
