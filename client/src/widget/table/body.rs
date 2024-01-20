use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;

use iced::{Element, Event, Length, Point, Rectangle, Size, touch};
use iced::advanced::{Clipboard, Layout, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, tree, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

pub struct Body<'a, M, R, F> {
  spacing: f32,
  column_count: usize,
  row_height: f32,
  row_height_plus_spacing: f32,
  row_count: usize,
  last_row_index: usize,
  cell_to_element: F,
  phantom_row: Element<'a, M, R>,
  element_state: RefCell<ElementState<'a, M, R>>,
}
impl<'a, M, R, F> Body<'a, M, R, F> {
  pub fn new(
    spacing: f32,
    column_count: usize,
    row_height: f32,
    row_count: usize,
    cell_to_element: F,
    phantom_row: Element<'a, M, R>
  ) -> Self {
    Self {
      spacing,
      column_count,
      row_height,
      row_height_plus_spacing: row_height + spacing,
      row_count,
      last_row_index: row_count.saturating_sub(1),
      cell_to_element,
      phantom_row,
      element_state: Default::default()
    }
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

impl<'a, F, M, R: Renderer> Widget<M, R> for Body<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn tag(&self) -> tree::Tag {
    tree::Tag::of::<RefCell<TreeState>>()
  }
  fn state(&self) -> tree::State {
    tree::State::Some(Box::new(RefCell::new(TreeState::default())))
  }
  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.phantom_row)]
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(std::slice::from_ref(&self.phantom_row))
  }

  fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fill) }
  fn layout(&self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
    let max_height = self.row_count as f32 * self.row_height + self.row_count.saturating_sub(1) as f32 * self.spacing;
    let limits = limits.max_height(max_height);
    // The phantom row lays out the cells of a single row. We will re-use that layout for every row in the table body,
    // but corrects its y-position to correspond to the actual row.
    let node = self.phantom_row.as_widget().layout(&mut tree.children[0], renderer, &limits.height(self.row_height));
    Node::with_children(limits.max(), vec![node])
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
    if self.row_count == 0 {
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
      let end = end.min(self.row_count); // Can't be longer than number of rows.
      start..end
    };

    // Remove trees and elements from rows that are no longer visible.
    let previous_rows = tree_state.previous_rows.clone();
    if previous_rows.start < rows.start {
      for row in previous_rows.start..rows.start.min(previous_rows.end) {
        element_state.remove_row(row, self.column_count);
        tree_state.remove_row(row, self.column_count);
      }
    }
    if previous_rows.end > rows.end {
      for row in rows.end.max(previous_rows.start)..previous_rows.end {
        element_state.remove_row(row, self.column_count);
        tree_state.remove_row(row, self.column_count);
      }
    }

    // Draw all table cells.
    for row in rows.clone() {
      for (col, cell_bounds) in (0..self.column_count).zip(Self::get_cell_bounds(layout)) {
        let cell = self.cell_at(
          row,
          col,
          cell_bounds,
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
        layout,
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
        layout,
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

impl<'a, F, M, R: Renderer> Body<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  /// Gets the cell at (`row`, `col`), with `cell_bounds` (retrieved from the layout of the phantom row).
  fn cell_at<'c>(
    &'c self,
    row: usize,
    col: usize,
    cell_bounds: Rectangle,
    absolute_y: f32,
    renderer: &R,
    element_state: &'c mut ElementState<'a, M, R>,
    tree_state: &'c mut TreeState,
  ) -> Cell<'c, 'a, M, R> {
    let element = element_state.get_or_insert(row, col, &self.cell_to_element);
    let tree = tree_state.get_or_insert(row, col, element);
    tree.diff(element.as_widget());
    let limits = Limits::new(Size::ZERO, cell_bounds.size());
    // Since `cell_bounds` is from the layout of the phantom row, it always has a y-position of 0.0. We move the node to
    // its correct y-position here.
    let y = absolute_y + row as f32 * self.row_height_plus_spacing;
    let node = element.as_widget()
      .layout(tree, renderer, &limits)
      .move_to(Point::new(cell_bounds.x, y));
    Cell { element, tree, node }
  }
  /// Gets the cell at `position` relative to this table, or `None` if there is no cell at `position`.
  fn cell_at_position<'c>(
    &'c self,
    position: Point,
    layout: Layout,
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
  /// Gets the column and bounds (retrieved from the layout of the phantom row) for `x` position relative to this table, or
  /// `None` if there is no column at `x`.
  fn col_and_bounds_at(&self, x: f32, layout: Layout) -> Option<(usize, Rectangle)> {
    // TODO: more efficient way to implement this, not a for loop?
    if x < 0.0 { return None; } // Out of bounds
    let mut offset = 0f32;
    for (col, bounds) in Self::get_cell_bounds(layout).enumerate() {
      if x < offset { return None; } // On column spacing or out of bounds
      offset += bounds.width;
      if x <= offset { return Some((col, bounds)); }
      offset += self.spacing;
    }
    None
  }
  /// Gets cell bounds (retrieved from the layout of the phantom row) from the `layout` of this table.
  #[inline]
  fn get_cell_bounds(layout: Layout) -> impl Iterator<Item=Rectangle> + '_ {
    layout.children().next().unwrap().children().map(|l| l.bounds())
  }
}

impl<'a, F, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for Body<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}

fn relative_to(point: Point, absolute: Point) -> Point {
  Point::new(point.x - absolute.x, point.y - absolute.y)
}
