use iced::advanced::Renderer;
use iced::Element;

pub mod builder;

pub mod child;

pub mod constrained_row;
pub mod modal;
pub mod table;
pub mod dark_light_toggle;
pub mod font;
pub mod icon;

/// Widget extensions
pub trait WidgetExt<'a, M, T, R> {
  fn into_element(self) -> Element<'a, M, T, R>;
}
impl<'a, M, T, R, W> WidgetExt<'a, M, T, R> for W where
  M: 'a,
  R: Renderer + 'a,
  W: Into<Element<'a, M, T, R>>
{
  #[inline]
  fn into_element(self) -> Element<'a, M, T, R> {
    self.into()
  }
}


