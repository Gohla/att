use iced::{Element, Length};
use iced::advanced::Renderer;
use iced::widget::{Column, scrollable, Scrollable, Space};

use crate::widget::constrained_row;
use crate::widget::constrained_row::ConstrainedRow;
use crate::widget::table::body::Body;

mod body;

pub struct Table<'a, M, R, F> {
  spacing: f32,
  width: Length,
  height: Length,
  max_width: f32,

  column_constraints: Vec<constrained_row::RowConstraint>,

  header_elements: Vec<Element<'a, M, R>>,
  header_row_height: f32,

  body_row_count: usize,
  body_row_height: f32,
  cell_to_element: F,
}

impl<'a, M, R, F> Table<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  pub fn new(cell_to_element: F) -> Self {
    Self::with_constraints_and_header_elements(Vec::new(), Vec::new(), cell_to_element)
  }
  pub fn with_constraints_and_header_elements(
    mut constraints: Vec<constrained_row::RowConstraint>,
    header_elements: Vec<Element<'a, M, R>>,
    cell_to_element: F,
  ) -> Self {
    constraints.resize_with(header_elements.len(), Default::default);
    let row_height = 26.0;
    Self {
      spacing: 1.0,
      width: Length::Fill,
      height: Length::Fill,
      max_width: f32::INFINITY,
      column_constraints: Vec::new(),
      header_elements: Vec::new(),
      header_row_height: row_height,
      body_row_count: 0,
      body_row_height: row_height,
      cell_to_element
    }
  }
  pub fn with_capacity(capacity: usize, cell_to_element: F) -> Self {
    Self::with_constraints_and_header_elements(Vec::with_capacity(capacity), Vec::with_capacity(capacity), cell_to_element)
  }

  pub fn spacing(mut self, spacing: f32) -> Self {
    self.spacing = spacing;
    self
  }
  pub fn width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }
  pub fn height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }
  pub fn max_width(mut self, max_width: f32) -> Self {
    self.max_width = max_width;
    self
  }

  pub fn header_row_height(mut self, height: f32) -> Self {
    self.header_row_height = height;
    self
  }

  pub fn body_row_count(mut self, body_row_count: usize) -> Self {
    self.body_row_count = body_row_count;
    self
  }
  pub fn body_row_height(mut self, height: f32) -> Self {
    self.body_row_height = height;
    self
  }

  pub fn push(mut self, column_constraint: impl Into<constrained_row::RowConstraint>, header_element: impl Into<Element<'a, M, R>>) -> Self {
    self.column_constraints.push(column_constraint.into());
    self.header_elements.push(header_element.into());
    self
  }
}

impl<'a, F, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for Table<'a, M, R, F> where
  R::Theme: scrollable::StyleSheet,
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn into(self) -> Element<'a, M, R> {
    let mut header = ConstrainedRow::with_elements_and_constraints(self.header_elements, self.column_constraints.clone());
    header.spacing = self.spacing;
    header.height = self.header_row_height;

    let column_count = self.column_constraints.len();
    // Create a phantom row with space elements which the table body widget will use as a base to lay out rows.
    let mut space_elements = Vec::with_capacity(column_count);
    space_elements.resize_with(column_count, || Space::new(Length::Fill, Length::Fill).into());
    let phantom_row = ConstrainedRow::with_elements_and_constraints(space_elements, self.column_constraints);

    let body = Body::new(self.spacing, column_count, self.body_row_height, self.body_row_count, self.cell_to_element, phantom_row.into());
    let body = Scrollable::new(body);

    Column::with_children(vec![header.into(), body.into()])
      .spacing(self.spacing)
      .width(self.width)
      .height(self.height)
      .max_width(self.max_width)
      .into()
  }
}

