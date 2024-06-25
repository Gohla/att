use iced::{Element, Font};
use iced::alignment::{Horizontal, Vertical};

use iced_builder::WidgetBuilder;

/// [Bootstrap icon](https://icons.getbootstrap.com/) font bytes.
pub const FONT_BYTES: &[u8] = include_bytes!("../../font/bootstrap-icons.ttf");
/// [Bootstrap icon](https://icons.getbootstrap.com/) font.
pub const FONT: Font = Font::with_name("bootstrap-icons");

pub fn icon_text<'a, M: 'a>(icon: &'static str) -> Element<'a, M> {
  WidgetBuilder::once()
    .text(icon)
    .font(FONT)
    .horizontal_alignment(Horizontal::Center)
    .vertical_alignment(Vertical::Center)
    .line_height(1.0)
    .add()
}

pub fn icon_button<'a, M: 'a>(icon: &'static str, on_press: impl Fn() -> M + 'a) -> Element<'a, M> {
  WidgetBuilder::once()
    .button(icon_text(icon))
    .on_press(on_press)
    .add()
}
