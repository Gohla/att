#![allow(dead_code)]
#![allow(unused_qualifications)]

use iced::{Element, Length, Point, Size};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::Renderer;
use iced::advanced::widget::Tree;
use iced::widget::{scrollable, Scrollable};

use crate::widget::table::header::TableHeader;
use crate::widget::table::rows::TableRows;
use crate::widget::table::widget::Table;

mod header;
mod rows;
mod widget;

//
// Table builder
//

// OPTO: Instead of rendering rows, render columns. Right now the mapper functions are dynamic dispatch because they
//       have different types, and we call the mapper on each cell. If we render columns instead, we only need one
//       dynamic dispatch per column. We can then also turn `&T` into `T` on the mapper. We do have to iterate over the
//       rows multiple times though, but this is possible because it is `Clone`. It might be a little bit slow because
//       `skip` on `Iterator` could be slow.

pub struct TableBuilder<'a, M, R, F> {
  width: Length,
  height: Length,
  max_width: f32,
  max_height: f32,
  spacing: f32,
  header: TableHeader<'a, M, R>,
  rows: TableRows<'a, M, R, F>,
}

impl<'a, M, R, F> TableBuilder<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  pub fn new(num_rows: usize, cell_to_element: F) -> Self {
    let spacing = 0.0;
    let row_height = 26.0;
    Self {
      width: Length::Fill,
      height: Length::Fill,
      max_width: f32::INFINITY,
      max_height: f32::INFINITY,
      spacing,
      header: TableHeader::new(spacing, row_height),
      rows: TableRows::new(spacing, row_height, num_rows, cell_to_element),
    }
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
  pub fn max_height(mut self, max_height: f32) -> Self {
    self.max_height = max_height;
    self
  }
  pub fn spacing(mut self, spacing: f32) -> Self {
    self.spacing = spacing;
    self.header.spacing = spacing;
    self.rows.spacing(spacing);
    self
  }
  pub fn header_row_height(mut self, height: f32) -> Self {
    self.header.row_height = height;
    self
  }
  pub fn row_height(mut self, height: f32) -> Self {
    self.rows.row_height(height);
    self
  }

  pub fn push_column(mut self, width_fill_portion: u32, header: impl Into<Element<'a, M, R>>) -> Self {
    self.header.push_column(width_fill_portion, header);
    self.rows.push_column(width_fill_portion);
    self
  }

  pub fn build(
    self,
  ) -> Table<'a, M, R> where
    M: 'a,
    R: Renderer + 'a,
    R::Theme: scrollable::StyleSheet
  {
    let rows = Scrollable::new(self.rows);
    Table::new(
      self.width,
      self.height,
      self.max_width,
      self.max_height,
      self.spacing,
      self.header.into(),
      rows.into(),
    )
  }
}


//
// Column layout calculation and reconstruction.
//

fn layout_columns<M, R: Renderer>(
  max_width: f32,
  row_height: f32,
  spacing: f32,
  width_fill_portions: &[u32],
  layout_elements: Option<(&[Element<'_, M, R>], &mut [Tree], &R)>,
) -> Vec<Node> {
  let num_columns = width_fill_portions.len();
  let last_column_index = num_columns.saturating_sub(1);
  let total_spacing_width = spacing * last_column_index as f32;
  let available_non_spacing_width = max_width - total_spacing_width;
  let total_fill_portion = width_fill_portions.iter().sum::<u32>() as f32; // TODO: cache
  let mut layouts = Vec::with_capacity(num_columns);
  let mut x_offset = 0f32;
  if let Some((elements, trees, renderer)) = layout_elements {
    for (i, ((width_fill_portion, element), tree)) in width_fill_portions.iter().zip(elements).zip(trees).enumerate() {
      let width = (*width_fill_portion as f32 / total_fill_portion) * available_non_spacing_width;
      let limits = Limits::new(Size::ZERO, Size::new(width, row_height));
      let mut layout = element.as_widget().layout(tree, renderer, &limits);
      layout.move_to(Point::new(x_offset, 0f32));
      layouts.push(layout);
      x_offset += width;
      if i < last_column_index { // Don't add spacing after last column.
        x_offset += spacing;
      }
    }
  } else { // TODO: reduce code duplication with if branch?
    for (i, width_fill_portion) in width_fill_portions.iter().enumerate() {
      let width = (*width_fill_portion as f32 / total_fill_portion) * available_non_spacing_width;
      let mut layout = Node::new(Size::new(width, row_height));
      layout.move_to(Point::new(x_offset, 0f32));
      layouts.push(layout);
      x_offset += width;
      if i < last_column_index { // Don't add spacing after last column.
        x_offset += spacing;
      }
    }
  }
  layouts
}
