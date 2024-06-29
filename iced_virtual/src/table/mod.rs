use iced::{Element, Length};
use iced::advanced::Renderer;
use iced::widget::{Column, Scrollable, scrollable, Space};

use crate::constrained_row::ConstrainedRow;
use crate::constrained_row::Constraint;
use crate::table::body::Body;

mod body;

pub struct Table<'a, M, T, R, F> {
  spacing: f32,
  width: Length,
  height: Length,
  max_width: f32,

  column_constraints: Vec<Constraint>,

  header_elements: Vec<Element<'a, M, T, R>>,
  header_row_height: f32,

  body_row_height: f32,
  body_row_count: usize,
  cell_to_element: F,
}

impl<'a, M, T, R, F> Table<'a, M, T, R, F> where
  F: Fn(usize, usize) -> Option<Element<'a, M, T, R>> + 'a,
{
  /// Creates a new table with a `cell_to_element` function to lazily create widget elements for cells.
  pub fn new(cell_to_element: F) -> Self {
    Self::with_constraints_and_header_elements(Vec::new(), Vec::new(), cell_to_element)
  }

  /// Creates a new table with:
  ///
  /// - `column_constraints`: constraints for column widths and alignments of elements in a cell.
  /// - `header_elements`: widget elements to use in the header of the table.
  /// - `cell_to_element`: function to lazily create widget elements for cells.
  ///
  /// If `column_constraints` is smaller or larger than `header_elements`, `column_constraints` will be resized to be
  /// the same size as `header_elements`, adding default constraints if needed.
  pub fn with_constraints_and_header_elements(
    mut column_constraints: Vec<Constraint>,
    header_elements: Vec<Element<'a, M, T, R>>,
    cell_to_element: F,
  ) -> Self {
    column_constraints.resize_with(header_elements.len(), Default::default);
    let row_height = 24.0;
    Self {
      spacing: 1.0,
      width: Length::Fill,
      height: Length::Fill,
      max_width: f32::INFINITY,
      column_constraints,
      header_elements,
      header_row_height: row_height,
      body_row_height: row_height,
      body_row_count: 0,
      cell_to_element,
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

  pub fn body_row_height(mut self, height: f32) -> Self {
    self.body_row_height = height;
    self
  }
  pub fn body_row_count(mut self, body_row_count: usize) -> Self {
    self.body_row_count = body_row_count;
    self
  }

  pub fn push(mut self, column_constraint: impl Into<Constraint>, header_element: impl Into<Element<'a, M, T, R>>) -> Self {
    self.column_constraints.push(column_constraint.into());
    self.header_elements.push(header_element.into());
    self
  }
}

impl<'a, F, M, T, R> Into<Element<'a, M, T, R>> for Table<'a, M, T, R, F> where
  M: 'a,
  T: scrollable::Catalog + 'a,
  R: Renderer + 'a,
  F: Fn(usize, usize) -> Option<Element<'a, M, T, R>> + 'a,
{
  fn into(self) -> Element<'a, M, T, R> {
    let header = ConstrainedRow::with_constraints_and_elements(self.column_constraints.clone(), self.header_elements)
      .spacing(self.spacing)
      .height(self.header_row_height);

    let column_count = self.column_constraints.len();
    // Create a phantom row with space elements which the table body widget will use as a base to lay out rows.
    let mut space_elements = Vec::with_capacity(column_count);
    space_elements.resize_with(column_count, || Space::new(Length::Fill, Length::Fill).into());
    let phantom_row = ConstrainedRow::with_constraints_and_elements(self.column_constraints, space_elements);

    let cell_to_element = move |row, col| (self.cell_to_element)(row, col)
      .unwrap_or_else(|| Space::new(Length::Fill, Length::Fill).into());
    let body = Body::new(self.spacing, column_count, self.body_row_height, self.body_row_count, cell_to_element, phantom_row.into());
    let body = Scrollable::new(body);

    Column::from_vec(vec![header.into(), body.into()])
      .spacing(self.spacing)
      .width(self.width)
      .height(self.height)
      .max_width(self.max_width)
      .into()
  }
}

