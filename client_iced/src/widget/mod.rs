use iced::advanced::Renderer;
use iced::Element;

pub mod builder;

pub mod child;

pub mod constrained_row;
pub mod modal;
pub mod table;
pub mod dark_light_toggle;
pub mod font;

/// Widget extensions
pub trait WidgetExt<'a, M, R> {
  fn into_element(self) -> Element<'a, M, R>;
}
impl<'a, M: 'a, R: Renderer + 'a, W: Into<Element<'a, M, R>>> WidgetExt<'a, M, R> for W {
  #[inline]
  fn into_element(self) -> Element<'a, M, R> {
    self.into()
  }
}


