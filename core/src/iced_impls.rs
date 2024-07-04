use iced::{Element, Font};
use iced::advanced::Renderer;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::Row;

use iced_builder::WidgetBuilder;
use iced_virtual::constrained_row::Constraint;
use iced_virtual::table::Table;

use crate::query::{FacetRef, FacetType, Query, QueryMessage};
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

/// Creates a table view for `service`, showing a `header` with `custom_buttons` and service actions, the query from the
/// service, and a table with the service's data.
///
/// Requests are converted to messages of type [M] with `map_request`, enabling `custom_buttons` to send custom messages.
/// Query messages are converted with `map_query_message` into [M].
pub fn as_full_table<'a, S: Service<Data: AsTableRow>, M: 'a>(
  service: &'a S,
  header: Option<&'a str>,
  custom_buttons: impl IntoIterator<Item=Element<'a, M>>,
  map_request: impl (Fn(S::Request) -> M) + 'a + Copy,
  map_query_message: impl (Fn(QueryMessage) -> M) + 'a + Copy,
) -> Element<'a, M> {
  let header = as_table_header(service, header, custom_buttons, map_request);
  let query = as_table_query(service).map(map_query_message);
  let table = as_table(service, map_request);
  let mut wb = WidgetBuilder::heap_with_capacity(3 + if header.is_some() { 2 } else { 0 });
  if let Some(header) = header {
    wb = wb
      .add_element(header)
      .add_horizontal_rule(1.0);
  }
  wb.add_element(query)
    .add_horizontal_rule(1.0)
    .add_element(table)
    .column().spacing(10.0).fill().add()
    .take()
}

/// Creates a table header for `service`, showing a `header` with `custom_buttons` and service actions.
///
/// Requests are converted to messages of type [M] with `map_request`, enabling `custom_buttons` to send custom messages.
pub fn as_table_header<'a, S: Service, M: 'a>(
  service: &'a S,
  header: Option<&'a str>,
  custom_buttons: impl IntoIterator<Item=Element<'a, M>>,
  map_request: impl (Fn(S::Request) -> M) + 'a + Copy,
) -> Option<Element<'a, M>> {
  let action_buttons = service.actions_with_definitions()
    .map(|action| action.into_element().map(map_request));
  let buttons: Vec<_> = custom_buttons.into_iter().chain(action_buttons).collect();

  let mut header_builder = WidgetBuilder::heap_with_capacity(3);
  if let Some(header) = header {
    header_builder = header_builder.text(header).size(20.0).add()
  }
  if !buttons.is_empty() {
    header_builder = header_builder.element(Row::from_vec(buttons).spacing(5.0)).add()
  };

  if !header_builder.is_empty() {
    let element = header_builder
      .add_space_fill_width()
      .row().spacing(10.0).align_center().fill_width().add()
      .take();
    Some(element)
  } else {
    None
  }
}

/// Creates a table query for `service`.
pub fn as_table_query<S: Service>(service: &S) -> Element<QueryMessage> {
  view_query(service.query())
}

/// Creates a table showing `service`'s data. Requests are converted to a message of type [M] with `map_request`.
pub fn as_table<'a, S: Service<Data: AsTableRow>, M: 'a>(
  service: &'a S,
  map_request: impl (Fn(S::Request) -> M) + 'a + Copy,
) -> Element<'a, M> {
  let cell_to_element = move |row, col| -> Option<Element<M>> {
    let Some(krate) = service.get_data(row) else { return None; };
    if let Some(text) = krate.cell(col as u8) {
      return Some(WidgetBuilder::once().add_text(text))
    }

    let action_index = col - S::Data::COLUMNS.len();
    let element = if let Some(action) = service.data_action_with_definition(action_index, krate) {
      action.into_element().map(map_request)
    } else {
      return None
    };
    Some(element)
  };

  let num_cols = S::Data::COLUMNS.len() + service.data_action_definitions().len();
  let mut table = Table::with_capacity(num_cols, cell_to_element)
    .spacing(1.0)
    .body_row_height(24.0)
    .body_row_count(service.data_len());
  for column in S::Data::COLUMNS {
    table = table.push(Constraint::new(column.width_fill_portion, column.horizontal_alignment.into(), column.vertical_alignment.into()), column.header)
  }
  for _ in service.data_action_definitions() {
    table = table.push(0.2, "");
  }

  table.into_element()
}

pub fn view_query<Q: Query>(query: &Q) -> Element<QueryMessage> {
  let num_facets = Q::FACET_DEFS.len();
  // Label text element + actual element + space element between elements.
  let capacity = num_facets * 2 + num_facets.saturating_sub(1);
  let mut builder = WidgetBuilder::heap_with_capacity(capacity);

  let mut first = true;
  for (facet_index, facet_def) in Q::FACET_DEFS.iter().enumerate() {
    let facet_index = facet_index as u8;
    let facet = query.facet(facet_index);

    if !first {
      // Spacing does not work with nested rows for some reason. Hack around it by adding space between elements.
      builder = builder.space().width(5.0).add();
    }
    first = false;

    builder = builder.text(format!("{}:", facet_def.label)).add();

    match &facet_def.facet_type { // TODO: create combined facet type + facet value for more type safety?
      FacetType::Boolean { default_value } => {
        let is_toggled = facet.map(FacetRef::into_bool)
          .transpose().unwrap_or_else(|f| panic!("facet {:?} at index {} is not a boolean", f, facet_index))
          .or(*default_value)
          .unwrap_or_default();
        let toggle_fn = move |toggled| QueryMessage::facet_change_bool(facet_index, toggled);
        builder = builder.toggler(None::<String>, is_toggled, toggle_fn)
          .spacing(0)
          .width_shrink()
          .add();
      }
      FacetType::String { default_value, placeholder } => {
        let text = facet.map(FacetRef::into_str)
          .transpose().unwrap_or_else(|f| panic!("facet {:?} at index {} is not a string", f, facet_index))
          .or(default_value.as_deref())
          .unwrap_or_default();
        builder = builder.text_input(placeholder.unwrap_or_default(), text)
          .on_input(move |text| QueryMessage::facet_change_string(facet_index, text))
          .add();
      }
    }
  }

  builder
    .row().spacing(5.0).align_center().fill_width().add()
    .take()
}
