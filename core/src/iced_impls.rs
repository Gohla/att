use iced::{Element, Font};
use iced::alignment::{Alignment, Horizontal, Vertical};

use iced_builder::WidgetBuilder;

use crate::service::{Action, ActionStyle, ActionWithDef};

impl From<crate::table::Alignment> for Alignment {
  fn from(alignment: crate::table::Alignment) -> Self {
    match alignment {
      crate::table::Alignment::Start => Alignment::Start,
      crate::table::Alignment::Center => Alignment::Center,
      crate::table::Alignment::End => Alignment::End,
    }
  }
}

impl<'a, A> From<ActionWithDef<'a, A>> for Element<'a, A::Request> where
  A: Action + 'a,
{
  fn from(ActionWithDef { definition, action }: ActionWithDef<A>) -> Self {
    let mut content = WidgetBuilder::once().text(definition.text);
    if let Some(font_name) = definition.font_name {
      content = content.font(Font::with_name(font_name));
    }
    if definition.icon {
      content = content
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .line_height(1.0)
    }
    let content: Element<'a, ()> = content.add();

    let mut button = WidgetBuilder::once()
      .button(content)
      .disabled(action.is_disabled())
      .on_press(move || action.request())
      ;
    if definition.icon {
      button = button.padding(3.0);
    }
    button = match definition.style {
      ActionStyle::Primary => button.primary_style(),
      ActionStyle::Secondary => button.secondary_style(),
      ActionStyle::Success => button.success_style(),
      ActionStyle::Danger => button.danger_style(),
    };
    button.add()
  }
}
