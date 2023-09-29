use iced::alignment::Horizontal;
use iced::Element;

use crate::widget::builder::WidgetBuilder;
use crate::widget::ICON_FONT;

pub fn light_dark_toggle<'a, M: 'a>(dark_mode_enabled: bool, on_press: impl Fn() -> M + 'a) -> Element<'a, M> {
  let icon = if dark_mode_enabled { "\u{f5a2}" } else { "\u{f496}" };
  let text = WidgetBuilder::once()
    .text(icon).font(ICON_FONT).horizontal_alignment(Horizontal::Center).width(20).add();
  WidgetBuilder::once()
    .button(text).on_press(on_press).add()
}
