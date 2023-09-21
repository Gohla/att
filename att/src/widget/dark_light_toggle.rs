use iced::alignment::Horizontal;
use iced::Element;
use iced::widget::{Button, Text};

use crate::widget::{ButtonEx, ICON_FONT};

pub fn light_dark_toggle<'a, M: 'a>(dark_mode_enabled: bool, on_press: impl Fn() -> M + 'a) -> Element<'a, M> {
  let icon = if dark_mode_enabled { "\u{f5a2}" } else { "\u{f496}" };
  let text = Text::new(icon)
    .font(ICON_FONT)
    .horizontal_alignment(Horizontal::Center)
    .width(20);
  Button::new(text)
    .on_press_into_element(on_press)
}
