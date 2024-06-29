use iced::{Element, Font};
use iced::advanced::Renderer;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::Row;

use iced_builder::WidgetBuilder;
use iced_virtual::constrained_row::Constraint;
use iced_virtual::table::Table;

use crate::crates::Crate;
use crate::query::{Facet, FacetType, Query, QueryDef};
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
  map_query_message: impl (Fn(QueryMessage) -> M) + 'a + Copy,
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
    .element(view_query(service.query_definition(), service.query())).map(map_query_message).add()
    .add_horizontal_rule(1.0)
    .add_element(table)
    .column().spacing(10.0).padding(10).fill().add()
    .take()
}

#[derive(Debug)]
pub enum QueryMessage {
  FacetChange {
    facet_id: &'static str,
    new_facet: Facet,
  }
}

pub fn view_query<'a>(query_def: &'a QueryDef, query: &'a Query) -> Element<'a, QueryMessage> {
  // Label text element + actual element + space element between elements.
  let capacity = query_def.facet_defs_len() * 2 + query_def.facet_defs_len().saturating_sub(1);
  let mut builder = WidgetBuilder::heap_with_capacity(capacity);

  let mut first = true;
  for (facet_id, facet_def, facet) in query.facets_with_defs(query_def) {
    if !first {
      // Spacing does not work with nested rows for some reason. Hack around it by adding space between elements.
      builder = builder.space().width(5.0).add();
    }
    first = false;

    builder = builder.text(format!("{}:", facet_def.label)).add();

    match &facet_def.facet_type { // TODO: create combined facet type + facet value for type safety
      FacetType::Boolean { default_value } => {
        let toggle_fn = |toggled| QueryMessage::FacetChange { facet_id, new_facet: Facet::new_boolean(toggled) };
        builder = builder.toggler(None::<String>, facet.as_bool().or(*default_value).unwrap_or_default(), toggle_fn)
          .spacing(0)
          .width_shrink()
          .add();
      }
      FacetType::String { default_value, placeholder } => {
        builder = builder.text_input(placeholder.unwrap_or_default(), facet.as_str().or(default_value.as_deref()).unwrap_or_default())
          .on_input(|text| QueryMessage::FacetChange { facet_id, new_facet: Facet::new_string(text) })
          .add();
      }
    }
  }

  builder
    .row().spacing(5.0).align_center().fill_width().add()
    .take()
}

pub fn update_query(query: &mut Query, message: QueryMessage) {
  match message {
    QueryMessage::FacetChange { facet_id, new_facet } => {
      if let Some(facet) = query.facet_mut(facet_id) {
        facet.set_from(new_facet);
      } else {
        panic!("facet '{}' not found in query {:?}", facet_id, query);
      }
    }
  }
}
