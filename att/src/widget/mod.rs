use iced::{Command, Element, Font};
use iced::advanced::{Renderer, Widget};
use iced::widget::button::{self, Button};
use iced_futures::MaybeSend;

pub mod modal;
pub mod dark_light_toggle;
pub mod builder;

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

/// [Bootstrap icon](https://icons.getbootstrap.com/) font. Only available after [`load_icon_font_command`] completes.
pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");

/// Create a command that loads the [Bootstrap icon font](ICON_FONT).
pub fn load_icon_font_command<M: 'static>(on_load: impl Fn(Result<(), iced::font::Error>) -> M + 'static + MaybeSend + Sync + Clone) -> Command<M> {
  const FONT_BYTES: &[u8] = include_bytes!("../../font/bootstrap-icons.ttf");
  iced::font::load(FONT_BYTES).map(on_load)
}
