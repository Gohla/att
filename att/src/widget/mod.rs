use iced::advanced::{Renderer, Widget};
use iced::Element;
use iced::widget::button::{self, Button};

pub mod modal;

/// Widget extensions
pub trait WidgetExt<'a, M, R> {
  fn into_element(self) -> Element<'a, M, R>;
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R>;
}
impl<'a, M: 'a, R: Renderer + 'a, W: Widget<M, R> + 'a> WidgetExt<'a, M, R> for W {
  #[inline]
  fn into_element(self) -> Element<'a, M, R> {
    Element::new(self)
  }
  #[inline]
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R> {
    self.into_element().map(f)
  }
}

/// Button widget extensions
pub trait ButtonEx<'a, R> {
  fn on_press_into_element<M: 'a, F: Fn() -> M + 'a>(self, f: F) -> Element<'a, M, R>;
}
impl<'a, R> ButtonEx<'a, R> for Button<'a, (), R> where
  R: Renderer + 'a,
  R::Theme: button::StyleSheet,
{
  fn on_press_into_element<M: 'a, F: Fn() -> M + 'a>(self, f: F) -> Element<'a, M, R> {
    self.on_press(()).map_into_element(move |_| f())
  }
}

/// Copy of column! macro, which the Rust plugin does not like due to the built-in column! macro.
macro_rules! col {
  () => (
    iced::widget::Column::new()
  );
  ($($x:expr),+ $(,)?) => (
    iced::widget::Column::with_children(vec![$(iced::Element::from($x)),+])
  );
}
pub(crate) use col;
