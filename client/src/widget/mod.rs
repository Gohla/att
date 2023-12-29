use iced::{Element, Font};
use iced::advanced::Renderer;

pub mod builder;

pub mod child;

pub mod constrained_row;
pub mod modal;
pub mod table;
pub mod dark_light_toggle;
pub mod maybe_send;

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

/// [Bootstrap icon](https://icons.getbootstrap.com/) font bytes.
pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../font/bootstrap-icons.ttf");
/// [Bootstrap icon](https://icons.getbootstrap.com/) font.
pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");
