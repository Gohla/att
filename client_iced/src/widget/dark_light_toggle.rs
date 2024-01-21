use iced::Element;

use crate::widget::icon::icon_button;

pub fn light_dark_toggle<'a, M: 'a>(dark_mode_enabled: bool, on_press: impl Fn() -> M + 'a) -> Element<'a, M> {
  let icon = if dark_mode_enabled { "\u{f5a2}" } else { "\u{f496}" };
  icon_button(icon, on_press)
}
