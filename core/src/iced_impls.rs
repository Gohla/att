use iced::{Element, Font};
use iced::advanced::Renderer;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::Row;

use iced_builder::WidgetBuilder;
use iced_virtual::constrained_row::Constraint;
use iced_virtual::table::Table;

use crate::crates::Crate;
use crate::service::{Action, ActionStyle, ActionWithDef, Service};
use crate::table::AsTableRow;

trait IntoElement<'a, M, T, R> {
  fn into_element(self) -> Element<'a, M, T, R>;
}
impl<'a, M, T, R, I> IntoElement<'a, M, T, R> for I where
  M: 'a,
  R: Renderer + 'a,
  I: Into<Element<'a, M, T, R>>
{
  #[inline]
  fn into_element(self) -> Element<'a, M, T, R> { self.into() }
}

impl From<crate::table::Alignment> for Alignment {
  fn from(alignment: crate::table::Alignment) -> Self {
    match alignment {
      crate::table::Alignment::Start => Alignment::Start,
      crate::table::Alignment::Center => Alignment::Center,
      crate::table::Alignment::End => Alignment::End,
    }
  }
}

impl<'a, A: Action + 'a> From<ActionWithDef<'a, A>> for Element<'a, A::Request> {
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

/// Creates a table view for `service`, showing a `header` with `custom_buttons` and service actions, and a table with
/// the service's data.
///
/// Requests are converted to a message with `map_request`, allowing `custom_buttons` to send custom messages.
pub fn as_table<'a, S: Service<Data: AsTableRow>, M: 'a>(
  service: &'a S,
  header: &'a str,
  map_request: impl (Fn(S::Request) -> M) + 'a + Copy,
  custom_buttons: impl IntoIterator<Item=Element<'a, M>>
) -> Element<'a, M> {
  let cell_to_element = move |row, col| -> Option<Element<M>> {
    let Some(krate) = service.get_data(row) else { return None; };
    if let Some(text) = krate.cell(col as u8) {
      return Some(WidgetBuilder::once().add_text(text))
    }

    let action_index = col - Crate::COLUMNS.len();
    let element = if let Some(action) = service.data_action_with_definition(action_index, krate) {
      action.into_element().map(map_request)
    } else {
      return None
    };
    Some(element)
  };
  let mut table = Table::with_capacity(5, cell_to_element)
    .spacing(1.0)
    .body_row_height(24.0)
    .body_row_count(service.data_len());
  for column in Crate::COLUMNS {
    table = table.push(Constraint::new(column.width_fill_portion, column.horizontal_alignment.into(), column.vertical_alignment.into()), column.header)
  }
  for _ in service.data_action_definitions() {
    table = table.push(0.2, "");
  }
  let table = table.into_element();

  let action_buttons = service.actions_with_definitions()
    .map(|action| action.into_element().map(map_request));
  let buttons: Vec<_> = custom_buttons.into_iter().chain(action_buttons).collect();

  WidgetBuilder::stack()
    .text(header).size(20.0).add()
    .add_element(Row::from_vec(buttons).spacing(5.0))
    .add_space_fill_width()
    .row().spacing(10.0).align_center().fill_width().add()
    .add_horizontal_rule(1.0)
    .add_element(table)
    .column().spacing(10.0).padding(10).fill().add()
    .take()
}
