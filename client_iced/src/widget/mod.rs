use iced::advanced::Renderer;
use iced::Element;

pub mod modal;
pub mod dark_light_toggle;
pub mod font;
pub mod icon;

/// Into element conversion.
pub trait IntoElement<'a, M, T, R> {
  fn into_element(self) -> Element<'a, M, T, R>;
}
impl<'a, M: 'a, T, R: Renderer + 'a, I: Into<Element<'a, M, T, R>>> IntoElement<'a, M, T, R> for I {
  #[inline]
  fn into_element(self) -> Element<'a, M, T, R> { self.into() }
}
