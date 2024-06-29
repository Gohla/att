use iced::advanced::Renderer;
use iced::Element;

pub mod modal;
pub mod font;
pub mod icon;

/// Conversion into an [`Element`]. So we don't have to disambiguate `widget.into()` calls.
pub trait IntoElement<'a, M, T, R> {
  /// Convert `self` into an [`Element`].
  fn into_element(self) -> Element<'a, M, T, R>;
}
impl<'a, M, T, R, I> IntoElement<'a, M, T, R> for I where
  M: 'a,
  R: Renderer + 'a,
  I: Into<Element<'a, M, T, R>>
{
  #[inline]
  fn into_element(self) -> Element<'a, M, T, R> { self.into() }
}
