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
  pub fn get_or_insert<F>(
    &mut self,
    row_index: usize,
    column_index: usize,
    cell_to_element: &F
  ) -> &mut Element<'a, M, R> where
    F: Fn(usize, usize) -> Element<'a, M, R> + 'a
  {
    self.elements.entry((row_index, column_index))
      .or_insert_with(|| cell_to_element(row_index, column_index))
  }
  pub fn remove_row(&mut self, row_index: usize, num_columns: usize) {
    for column_index in 0..num_columns {
      self.elements.remove(&(row_index, column_index));
    }
  }
}

#[derive(Default)]
struct TreeState {
  trees: HashMap<(usize, usize), Tree>,
  previous_row_indices: Range<usize>,
}
impl TreeState {
  pub fn get_or_insert<'a, M, R: Renderer>(
    &mut self,
    row_index: usize,
    column_index: usize,
    element: &Element<'a, M, R>
  ) -> &mut Tree {
    self.trees.entry((row_index, column_index))
      .or_insert_with(|| Tree::new(element))
  }
  pub fn remove_row(&mut self, row_index: usize, num_columns: usize) {
    for column_index in 0..num_columns {
      self.trees.remove(&(row_index, column_index));
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
    let relative_y = viewport.y - absolute_y;

    // Row indices: calculate visible rows.
    let row_indices = {
      let start = relative_y / self.row_height_plus_spacing;
      let start = start.floor() as usize; // Use floor so partial rows are visible.
      let start = start.min(self.last_row_index); // Can't start past last row.
      let length = viewport.height / self.row_height_plus_spacing;
      let length = length.ceil() as usize; // Use ceil so partial rows are visible.
      let end = start + length;
      let end = end.min(self.num_rows); // Can't be longer than number of rows.
      start..end
    };

    // Remove trees and elements from rows that are no longer visible.
    let prev_row_indices = tree_state.previous_row_indices.clone();
    if prev_row_indices.start < row_indices.start {
      let row_indices_to_delete = prev_row_indices.start..row_indices.start.min(prev_row_indices.end);
      for row_index in row_indices_to_delete {
        println!("Removing row {}", row_index);
        element_state.remove_row(row_index, self.num_columns);
        tree_state.remove_row(row_index, self.num_columns);
      }
    }
    if prev_row_indices.end > row_indices.end {
      let row_indices_to_delete = row_indices.end.max(prev_row_indices.start)..prev_row_indices.end;
      for row_index in row_indices_to_delete {
        println!("Removing row {}", row_index);
        element_state.remove_row(row_index, self.num_columns);
        tree_state.remove_row(row_index, self.num_columns);
      }
    }

    // Draw all table cells.
    let mut y = absolute_y + row_indices.start as f32 * self.row_height_plus_spacing;
    for row_index in row_indices.clone() {
      for (column_index, cell_layout) in (0..self.num_columns).into_iter().zip(layout.children()) {
        let element = element_state.get_or_insert(row_index, column_index, &self.cell_to_element);
        let tree = tree_state.get_or_insert(row_index, column_index, element);
        let bounds = cell_layout.bounds();
        let limits = Limits::new(Size::ZERO, bounds.size());
        let node = reconstruct_node(element, tree, renderer, &limits, Point::new(bounds.x, y));
        let layout = Layout::new(&node);
        element.as_widget().draw(&tree, renderer, theme, style, layout, cursor, viewport);
      }

      y += self.row_height;
      if row_index < self.last_row_index { // Don't add spacing after last row.
        y += self.spacing;
      }
    }

    // Store current row indices.
    tree_state.previous_row_indices = row_indices;
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
    let absolute_position = layout.position();
    let absolute_y = absolute_position.y;
    let cursor_position = cursor.position();

    let result = match &event {
      Event::Mouse(_) => {
        if let Some(cursor_position) = cursor_position {
          let position = relative_to(cursor_position, absolute_position);
          self.row_column_index_layout_at(position, &layout)
        } else {
          None
        }
      }
      Event::Touch(touch_event) => {
        let element_position = match touch_event {
          touch::Event::FingerPressed { position, .. } => position,
          touch::Event::FingerMoved { position, .. } => position,
          touch::Event::FingerLifted { position, .. } => position,
          touch::Event::FingerLost { position, .. } => position,
        };
        let position = relative_to(*element_position, absolute_position);
        self.row_column_index_layout_at(position, &layout)
      }
      // TODO: propagate other events?
      _ => None,
    };

    if let Some((row_index, column_index, cell_layout)) = result {
      let mut element_state = self.element_state.borrow_mut();
      let mut tree_state = tree.state.downcast_ref::<RefCell<TreeState>>().borrow_mut();

      let element = element_state.get_or_insert(row_index, column_index, &self.cell_to_element);
      let tree = tree_state.get_or_insert(row_index, column_index, element);
      let bounds = cell_layout.bounds();
      let limits = Limits::new(Size::ZERO, bounds.size());
      let y = absolute_y + row_index as f32 * self.row_height_plus_spacing;
      let node = reconstruct_node(element, tree, renderer, &limits, Point::new(bounds.x, y));
      let layout = Layout::new(&node);
      return element.as_widget_mut().on_event(
        tree,
        event,
        layout,
        cursor,
        renderer,
        clipboard,
        shell,
        viewport
      )
    }

    Status::Ignored
  }
  fn mouse_interaction(&self, _tree: &Tree, _layout: Layout, _cursor: Cursor, _viewport: &Rectangle, _renderer: &R) -> Interaction {
    Interaction::default()
    // TODO: implement
  }
  fn operate(&self, _tree: &mut Tree, _layout: Layout, _renderer: &R, _operation: &mut dyn Operation<M>) {
    // TODO: will operation propagation actually do anything because currently "virtual" widgets have no state?
    // TODO: implement
  }

  fn overlay<'o>(&'o mut self, _state: &'o mut Tree, _layout: Layout, _renderer: &R) -> Option<iced::advanced::overlay::Element<'a, M, R>> {
    // TODO: implement
    None
  }
}

impl<'a, F, M, R: Renderer> TableRows<'a, M, R, F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn row_index_at(&self, y: f32) -> Option<usize> {
    // TODO: return None when row index > num_rows!
    if y < 0.0 { return None; } // Out of bounds
    let row_offset = (y / self.row_height_plus_spacing).ceil() as usize;
    let row_offset_without_spacing = (row_offset as f32 * self.row_height_plus_spacing) - self.spacing;
    if y > row_offset_without_spacing {
      None // On row spacing
    } else {
      Some(row_offset.saturating_sub(1)) // NOTE: + 1 because row_offset is 1-based. Why is this the case? // TODO: investigate this
    }
  }
  fn column_index_and_layout_at<'l>(&self, x: f32, layout: &Layout<'l>) -> Option<(usize, Layout<'l>)> {
    // TODO: more efficient way to implement this, not a for loop!
    if x < 0.0 { return None; } // Out of bounds
    let mut offset = 0f32;
    for (column_index, cell_layout) in layout.children().enumerate() {
      if x < offset { return None; } // On column spacing or out of bounds
      offset += cell_layout.bounds().width;
      if x <= offset { return Some((column_index, cell_layout)); }
      offset += self.spacing;
    }
    None
  }
  fn row_column_index_layout_at<'l>(&self, position: Point, layout: &Layout<'l>) -> Option<(usize, usize, Layout<'l>)> {
    if let Some(row_index) = self.row_index_at(position.x) {
      if let Some((column_index, cell_layout)) = self.column_index_and_layout_at(position.x, layout) {
        return Some((row_index, column_index, cell_layout));
      }
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

/// Reconstruct layout node of `element`, offsetting its position by `translation`.
fn reconstruct_node<M, R: Renderer>(
  element: &Element<'_, M, R>,
  tree: &mut Tree,
  renderer: &R,
  limits: &Limits,
  position: Point,
) -> Node {
  // HACK: Reconstruct the layout from `limits` which has a correct x position, but an incorrect y position which always
  //       points to the first row. This is needed so that we do not have to lay out all the cells of the table each
  //       time the layout changes, because that is slow for larger tables.
  let mut node = element.as_widget().layout(tree, renderer, &limits);
  // Translate to fix the y position.
  node.move_to(position);
  node
}

fn relative_to(point: Point, absolute: Point) -> Point {
  Point::new(point.x - absolute.x, point.y - absolute.y)
}
