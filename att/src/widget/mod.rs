use iced::{Command, Element, Font};
use iced::advanced::Renderer;

use crate::widget::maybe_send::MaybeSend;

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

/// [Bootstrap icon](https://icons.getbootstrap.com/) font. Only available after [`load_icon_font_command`] completes.
pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");

/// Create a command that loads the [Bootstrap icon font](ICON_FONT).
pub fn load_icon_font_command<M: 'static>(on_load: impl Fn(Result<(), iced::font::Error>) -> M + 'static + MaybeSend + Sync + Clone) -> Command<M> {
  const FONT_BYTES: &[u8] = include_bytes!("../../font/bootstrap-icons.ttf");
  iced::font::load(FONT_BYTES).map(on_load)
}
