use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;

use iced::{Element, Event, Length, Point, Rectangle, Size, touch};
use iced::advanced::{Clipboard, Layout, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, tree, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

use crate::widget::table::layout_columns;

pub struct TableRows<'a, M, R, F> {
  spacing: f32,

  row_height: f32,
  row_height_plus_spacing: f32,
  num_rows: usize,
  last_row_index: usize,

  column_fill_portions: Vec<u32>,
  num_columns: usize,

  cell_to_element: F,
  element_state: RefCell<ElementState<'a, M, R>>,
}
impl<'a, M, R, F> TableRows<'a, M, R, F> {
  pub fn new(spacing: f32, row_height: f32, num_rows: usize, cell_to_element: F) -> Self {
    Self {
      spacing,

      row_height,
      row_height_plus_spacing: row_height + spacing,
      num_rows,
      last_row_index: num_rows.saturating_sub(1),

      num_columns: 0,
      column_fill_portions: Vec::new(),

      cell_to_element,
      element_state: Default::default()
    }
  }

  pub fn spacing(&mut self, spacing: f32) {
    self.spacing = spacing;
    self.row_height_plus_spacing = self.row_height + spacing;
  }
  pub fn row_height(&mut self, row_height: f32) {
    self.row_height = row_height;
    self.row_height_plus_spacing = row_height + self.spacing;
  }

  pub fn push_column(&mut self, column_fill_portion: u32) {
    self.column_fill_portions.push(column_fill_portion);
    self.num_columns += 1;
  }
}

struct ElementState<'a, M, R> {
  elements: HashMap<(usize, usize), Element<'a, M, R>>,
}
impl<'a, M, R> Default for ElementState<'a, M, R> {
  fn default() -> Self {
    Self { elements: Default::default(), }
  }
}
impl<'a, M, R> ElementState<'a, M, R> {
  pub fn get_or_insert<F>(&mut self, row: usize, col: usize, cell_to_element: &F) -> &mut Element<'a, M, R> where
    F: Fn(usize, usize) -> Element<'a, M, R> + 'a
  {
    self.elements.entry((row, col))
      .or_insert_with(|| cell_to_element(row, col))
  }
  pub fn remove_row(&mut self, row: usize, num_columns: usize) {
    for col in 0..num_columns {
      self.elements.remove(&(row, col));
    }
  }
}

#[derive(Default)]
struct TreeState {
  trees: HashMap<(usize, usize), Tree>,
  previous_rows: Range<usize>,
}
impl TreeState {
  pub fn get_or_insert<'a, M, R: Renderer>(&mut self, row: usize, col: usize, element: &Element<'a, M, R>) -> &mut Tree {
    self.trees.entry((row, col))
      .or_insert_with(|| Tree::new(element))
  }
  pub fn remove_row(&mut self, row: usize, num_columns: usize) {
    for col in 0..num_columns {
      self.trees.remove(&(row, col));
    }
  }
}


impl<'a, F, M, R: Renderer> Widget<M, R> for TableRows<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn tag(&self) -> tree::Tag { tree::Tag::of::<RefCell<TreeState>>() }
  fn state(&self) -> tree::State { tree::State::Some(Box::new(RefCell::new(TreeState::default()))) }
  fn children(&self) -> Vec<Tree> { Vec::new() }
  fn diff(&self, _tree: &mut Tree) {
    // TODO: implement
  }

  fn width(&self) -> Length { Length::Fill }
  fn height(&self) -> Length { Length::Fill }
  fn layout(&self, _tree: &mut Tree, _renderer: &R, limits: &Limits) -> Node {
    let available_width = limits.max().width;
    // HACK: only lay out first row, because laying out the entire table becomes slow for larger tables. Reconstruct
    //       the layout of elements on-demand with `reconstruct_layout_node`.
    let layouts = layout_columns::<M, R>(available_width, self.row_height, self.spacing, &self.column_fill_portions, None);
    let total_height = self.num_rows * self.row_height as usize + self.num_rows.saturating_sub(1) * self.spacing as usize;
    Node::with_children(Size::new(available_width, total_height as f32), layouts)
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut R,
    theme: &R::Theme,
    style: &renderer::Style,
    layout: Layout,
    cursor: Cursor,
    viewport: &Rectangle,
  ) {
    if self.num_rows == 0 {
      return;
    }

    let mut element_state = self.element_state.borrow_mut();
    let mut tree_state = tree.state.downcast_ref::<RefCell<TreeState>>().borrow_mut();

    let absolute_y = layout.position().y;
    let y = viewport.y - absolute_y;

    // Calculate visible rows.
    let rows = {
      let start = y / self.row_height_plus_spacing;
      let start = start.max(0.0); // Can't start on negative row.
      let start_floored = start.floor(); // Use floor so partial rows are visible.
      let floored_amount = start - start_floored; // Store how much we floored off for length calculation.
      let start = start_floored as usize;
      let start = start.min(self.last_row_index); // Can't start past last row.

      // Use floored amount to account for extra space at the bottom in which an additional row can be visible.
      let additional_height = floored_amount * self.row_height_plus_spacing;
      let length = (viewport.height + additional_height) / self.row_height_plus_spacing;
      let length = length.ceil() as usize; // Use ceil so partial rows are visible.

      let end = start + length;
      let end = end.min(self.num_rows); // Can't be longer than number of rows.
      start..end
    };

    // Remove trees and elements from rows that are no longer visible.
    let previous_rows = tree_state.previous_rows.clone();
    if previous_rows.start < rows.start {
      for row in previous_rows.start..rows.start.min(previous_rows.end) {
        element_state.remove_row(row, self.num_columns);
        tree_state.remove_row(row, self.num_columns);
      }
    }
    if previous_rows.end > rows.end {
      for row in rows.end.max(previous_rows.start)..previous_rows.end {
        element_state.remove_row(row, self.num_columns);
        tree_state.remove_row(row, self.num_columns);
      }
    }

    // Draw all table cells.
    for row in rows.clone() {
      for (col, cell_layout) in (0..self.num_columns).into_iter().zip(layout.children()) {
        let cell = self.cell_at(
          row,
          col,
          cell_layout.bounds(),
          absolute_y,
          renderer,
          &mut element_state,
          &mut tree_state
        );
        cell.element.as_widget().draw(
          cell.tree,
          renderer,
          theme,
          style,
          Layout::new(&cell.node),
          cursor,
          viewport
        );
      }
    }

    // Store current row indices.
    tree_state.previous_rows = rows;
  }

  fn on_event(
    &mut self,
    tree: &mut Tree,
    event: Event,
    layout: Layout,
    cursor: Cursor,
    renderer: &R,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, M>,
    viewport: &Rectangle,
  ) -> Status {
    let event_position = match &event {
      Event::Mouse(_) => {
        cursor.position()
      }
      Event::Touch(touch_event) => {
        let touch_position = match touch_event {
          touch::Event::FingerPressed { position, .. } => position,
          touch::Event::FingerMoved { position, .. } => position,
          touch::Event::FingerLifted { position, .. } => position,
          touch::Event::FingerLost { position, .. } => position,
        };
        Some(*touch_position)
      }
      _ => None, // TODO: propagate other events?
    };

    if let Some(event_position) = event_position {
      let absolute_position = layout.position();
      let position = relative_to(event_position, absolute_position);
      let mut element_state = self.element_state.borrow_mut();
      let mut tree_state = tree.state.downcast_ref::<RefCell<TreeState>>().borrow_mut();
      if let Some(cell) = self.cell_at_position(
        position,
        &layout,
        absolute_position.y,
        renderer,
        &mut element_state,
        &mut tree_state
      ) {
        return cell.element.as_widget_mut().on_event(
          cell.tree,
          event,
          Layout::new(&cell.node),
          cursor,
          renderer,
          clipboard,
          shell,
          viewport
        );
      }
    }

    Status::Ignored
  }
  fn mouse_interaction(&self, tree: &Tree, layout: Layout, cursor: Cursor, viewport: &Rectangle, renderer: &R) -> Interaction {
    if let Some(cursor_position) = cursor.position() {
      let absolute_position = layout.position();
      let position = relative_to(cursor_position, absolute_position);
      let mut element_state = self.element_state.borrow_mut();
      let mut tree_state = tree.state.downcast_ref::<RefCell<TreeState>>().borrow_mut();
      if let Some(cell) = self.cell_at_position(
        position,
        &layout,
        absolute_position.y,
        renderer,
        &mut element_state,
        &mut tree_state
      ) {
        return cell.element.as_widget().mouse_interaction(
          cell.tree,
          Layout::new(&cell.node),
          cursor,
          viewport,
          renderer,
        );
      }
    }
    Interaction::default()
  }
  fn operate(&self, _tree: &mut Tree, _layout: Layout, _renderer: &R, _operation: &mut dyn Operation<M>) {
    // TODO: implement?
  }
}

struct Cell<'c, 'e, M, R> {
  element: &'c mut Element<'e, M, R>,
  tree: &'c mut Tree,
  node: Node,
}

impl<'a, F, M, R: Renderer> TableRows<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  /// Gets the cell at (`row`, `col`).
  fn cell_at<'c>(
    &'c self,
    row: usize,
    col: usize,
    bounds: Rectangle,
    absolute_y: f32,
    renderer: &R,
    element_state: &'c mut ElementState<'a, M, R>,
    tree_state: &'c mut TreeState,
  ) -> Cell<'c, 'a, M, R> {
    let element = element_state.get_or_insert(row, col, &self.cell_to_element);
    let tree = tree_state.get_or_insert(row, col, element);
    let limits = Limits::new(Size::ZERO, bounds.size());
    let y = absolute_y + row as f32 * self.row_height_plus_spacing;
    let mut node = element.as_widget().layout(tree, renderer, &limits);
    node.move_to(Point::new(bounds.x, y));
    Cell { element, tree, node }
  }
  /// Gets the cell at `position` relative to this table, or `None` if there is no cell at `position`.
  fn cell_at_position<'c>(
    &'c self,
    position: Point,
    layout: &Layout,
    absolute_y: f32,
    renderer: &R,
    element_state: &'c mut ElementState<'a, M, R>,
    tree_state: &'c mut TreeState,
  ) -> Option<Cell<'c, 'a, M, R>> {
    if let Some(row) = self.row_at(position.y) {
      if let Some((col, bounds)) = self.col_and_bounds_at(position.x, layout) {
        return Some(self.cell_at(row, col, bounds, absolute_y, renderer, element_state, tree_state));
      }
    }
    None
  }
  /// Gets the row for `y` position relative to this table, or `None` if there is now row at `y`.
  fn row_at(&self, y: f32) -> Option<usize> {
    if y < 0.0 { return None; } // Out of bounds
    let row = y / self.row_height_plus_spacing;
    if y > (row.ceil() * self.row_height_plus_spacing) - self.spacing {
      None // On row spacing
    } else {
      let row = row.floor() as usize;
      if row > self.last_row_index {
        None // Out of bounds
      } else {
        Some(row)
      }
    }
  }
  /// Gets the column and bounds for `x` position relative to this table, or `None` if there is now column at `y`.
  fn col_and_bounds_at(&self, x: f32, layout: &Layout) -> Option<(usize, Rectangle)> {
    // TODO: more efficient way to implement this, not a for loop!
    if x < 0.0 { return None; } // Out of bounds
    let mut offset = 0f32;
    for (col, cell_layout) in layout.children().enumerate() {
      if x < offset { return None; } // On column spacing or out of bounds
      offset += cell_layout.bounds().width;
      if x <= offset { return Some((col, cell_layout.bounds())); }
      offset += self.spacing;
    }
    None
  }
}

impl<'a, F, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for TableRows<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}

fn relative_to(point: Point, absolute: Point) -> Point {
  Point::new(point.x - absolute.x, point.y - absolute.y)
}
