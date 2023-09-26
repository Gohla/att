#![allow(dead_code)]

use std::cell::RefCell;
use iced::{Element, Event, Length, Point, Rectangle, Size, touch};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::widget::tree;
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};
use iced::widget::scrollable;

//
// Table builder
//

// OPTO: Instead of rendering rows, render columns. Right now the mapper functions are dynamic dispatch because they
//       have different types, and we call the mapper on each cell. If we render columns instead, we only need one
//       dynamic dispatch per column. We can then also turn `&T` into `T` on the mapper. We do have to iterate over the
//       rows multiple times though, but this is possible because it is `Clone`. It might be a little bit slow because
//       `skip` on `Iterator` could be slow.

pub struct TableBuilder<'a, F, M, R> {
  width: Length,
  height: Length,
  max_width: f32,
  max_height: f32,
  spacing: f32,
  header: TableHeader<'a, M, R>,
  rows: TableRows<F>,
}

impl<'a, F, M, R> TableBuilder<'a, F, M, R> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  pub fn new(num_rows: usize, mapper: F) -> Self {
    let spacing = 0.0;
    let row_height = 26.0;
    Self {
      width: Length::Fill,
      height: Length::Fill,
      max_width: f32::INFINITY,
      max_height: f32::INFINITY,
      spacing,
      header: TableHeader { spacing, row_height, width_fill_portions: Vec::new(), headers: Vec::new() },
      rows: TableRows { spacing, row_height, width_fill_portions: Vec::new(), num_rows, mapper, tree_cache: RefCell::new(Vec::new()) },
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
    self.rows.spacing = spacing;
    self
  }
  pub fn header_row_height(mut self, height: f32) -> Self {
    self.header.row_height = height;
    self
  }
  pub fn row_height(mut self, height: f32) -> Self {
    self.rows.row_height = height;
    self
  }

  pub fn push_column(mut self, width_fill_portion: u32, header: impl Into<Element<'a, M, R>>) -> Self {
    self.header.width_fill_portions.push(width_fill_portion);
    self.header.headers.push(header.into());
    self.rows.width_fill_portions.push(width_fill_portion);
    self
  }

  pub fn build(
    self,
  ) -> Table<'a, M, R> where
    M: 'a,
    R: Renderer + 'a,
    R::Theme: scrollable::StyleSheet
  {
    // TODO: enable scrollable again.
    // let rows = Scrollable::new(self.rows);
    let rows = self.rows;
    Table {
      width: self.width,
      height: self.height,
      max_width: self.max_width,
      max_height: self.max_height,
      spacing: self.spacing,
      header: self.header.into(),
      rows: rows.into(),
    }
  }
}

//
// Table widget
//

/// TODO: could this entire table widget just be replaced by a column with header and rows?

pub struct Table<'a, M, R> {
  width: Length,
  height: Length,
  max_width: f32,
  max_height: f32,
  spacing: f32,
  header: Element<'a, M, R>,
  rows: Element<'a, M, R>,
}

impl<'a, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for Table<'a, M, R> {
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}

impl<'a, M: 'a, R: Renderer + 'a> Widget<M, R> for Table<'a, M, R> {
  fn state(&self) -> tree::State { tree::State::None }
  fn tag(&self) -> tree::Tag { tree::Tag::stateless() }
  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.header), Tree::new(&self.rows)]
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&[&self.header, &self.rows])
  }

  fn width(&self) -> Length { self.width }
  fn height(&self) -> Length { self.height }
  fn layout(&self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
    let limits = limits
      .max_width(self.max_width)
      .max_height(self.max_height)
      .width(self.width)
      .height(self.height);

    let (header_tree, rows_tree, ) = Self::unfold_tree_mut(tree);

    let header_layout = self.header.as_widget().layout(header_tree, renderer, &limits);
    let header_size = header_layout.size();
    let header_y_offset = header_size.height + self.spacing;

    let limits = limits.shrink(Size::new(0f32, header_y_offset));
    let mut rows_layout = self.rows.as_widget().layout(rows_tree, renderer, &limits);
    rows_layout.move_to(Point::new(0f32, header_y_offset));
    let rows_size = rows_layout.size();

    let size = Size::new(rows_size.width.max(rows_size.width), header_size.height + self.spacing + rows_size.height);
    Node::with_children(size, vec![header_layout, rows_layout])
  }
  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout<'_>, renderer: &R) -> Option<overlay::Element<'o, M, R>> {
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree_mut(tree);
    if let Some(header_overlay) = self.header.as_widget_mut().overlay(header_tree, header_layout, renderer) {
      return Some(header_overlay)
    }
    self.rows.as_widget_mut().overlay(rows_tree, rows_layout, renderer)
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
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree_mut(tree);
    if let Status::Captured = self.header.as_widget_mut().on_event(header_tree, event.clone(), header_layout, cursor, renderer, clipboard, shell, viewport) {
      return Status::Captured;
    }
    self.rows.as_widget_mut().on_event(rows_tree, event, rows_layout, cursor, renderer, clipboard, shell, viewport)
  }
  fn operate(&self, tree: &mut Tree, layout: Layout, renderer: &R, operation: &mut dyn Operation<M>) {
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree_mut(tree);
    self.header.as_widget().operate(header_tree, header_layout, renderer, operation);
    self.rows.as_widget().operate(rows_tree, rows_layout, renderer, operation);
  }
  fn mouse_interaction(&self, tree: &Tree, layout: Layout, cursor: Cursor, viewport: &Rectangle, renderer: &R) -> Interaction {
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree(tree);
    let header_interaction = self.header.as_widget().mouse_interaction(header_tree, header_layout, cursor, viewport, renderer);
    let rows_interaction = self.rows.as_widget().mouse_interaction(rows_tree, rows_layout, cursor, viewport, renderer);
    header_interaction.max(rows_interaction)
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
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree(tree);
    self.header.as_widget().draw(header_tree, renderer, theme, style, header_layout, cursor, viewport);
    self.rows.as_widget().draw(rows_tree, renderer, theme, style, rows_layout, cursor, viewport);
  }
}

impl<'a, M: 'a, R: Renderer + 'a> Table<'a, M, R> {
  fn unfold_tree(tree: &Tree) -> (&Tree, &Tree) {
    (&tree.children[0], &tree.children[1])
  }
  fn unfold_tree_mut(tree: &mut Tree) -> (&mut Tree, &mut Tree) {
    let mut tree_iter = tree.children.iter_mut();
    (tree_iter.next().unwrap(), tree_iter.next().unwrap())
  }
  fn unfold_layout<'t>(layout: Layout) -> (Layout, Layout) {
    let mut layout_iter = layout.children();
    (layout_iter.next().unwrap(), layout_iter.next().unwrap())
  }
}


//
// Table header
//

impl<'a, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for TableHeader<'a, M, R> {
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}

pub struct TableHeader<'a, M, R> {
  spacing: f32,
  row_height: f32,
  width_fill_portions: Vec<u32>,
  headers: Vec<Element<'a, M, R>>,
}

impl<'a, M, R: Renderer> Widget<M, R> for TableHeader<'a, M, R> {
  fn state(&self) -> tree::State { tree::State::None }
  fn tag(&self) -> tree::Tag { tree::Tag::stateless() }
  fn children(&self) -> Vec<Tree> {
    self.headers.iter().map(Tree::new).collect()
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&self.headers);
  }

  fn width(&self) -> Length { Length::Fill }
  fn height(&self) -> Length { self.row_height.into() }
  fn layout(&self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
    let total_width = limits.max().width;
    let layouts = layout_columns(total_width, self.row_height, self.spacing, &self.width_fill_portions, &self.headers, &mut tree.children, renderer);
    Node::with_children(Size::new(total_width, self.row_height), layouts)
  }
  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout, renderer: &R) -> Option<overlay::Element<'o, M, R>> {
    overlay::from_children(&mut self.headers, tree, layout, renderer)
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
    on_event_to_children(&mut self.headers, tree, event, layout, cursor, renderer, clipboard, shell, viewport)
  }
  fn mouse_interaction(&self, tree: &Tree, layout: Layout, cursor: Cursor, viewport: &Rectangle, renderer: &R) -> Interaction {
    mouse_interaction_to_children(&self.headers, tree, layout, cursor, viewport, renderer)
  }
  fn operate(&self, tree: &mut Tree, layout: Layout, renderer: &R, operation: &mut dyn Operation<M>) {
    operate_to_children(&self.headers, tree, layout, renderer, operation)
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
    if self.headers.is_empty() {
      return;
    }
    draw_children(&self.headers, tree, renderer, theme, style, layout, cursor, viewport)
  }
}

//
// Table rows
//

struct TableRows<F> {
  spacing: f32,
  row_height: f32,
  width_fill_portions: Vec<u32>,
  num_rows: usize,
  mapper: F,
  tree_cache: RefCell<Vec<Tree>>,
}

impl<'a, F, M, R: Renderer> Into<Element<'a, M, R>> for TableRows<F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}

impl<'a, F, M, R: Renderer> Widget<M, R> for TableRows<F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn state(&self) -> tree::State { tree::State::None }
  fn tag(&self) -> tree::Tag { tree::Tag::stateless() }
  fn children(&self) -> Vec<Tree> { Vec::new() }
  fn diff(&self, _tree: &mut Tree) {}

  fn width(&self) -> Length { Length::Fill }
  fn height(&self) -> Length { Length::Fill }
  fn layout(&self, _tree: &mut Tree, _renderer: &R, limits: &Limits) -> Node {
    let limit_max = limits.max();
    let total_width = limit_max.width;
    // HACK: only lay out first row, because laying out the entire table becomes slow for larger tables. Reconstruct
    //       the layout of elements on-demand with `reconstruct_layout_node`.
    let layouts = layout_virtual_columns(total_width, self.row_height, self.spacing, &self.width_fill_portions);
    let total_height = self.num_rows * self.row_height as usize + self.num_rows.saturating_sub(1) * self.spacing as usize;
    Node::with_children(Size::new(total_width, total_height as f32), layouts)
  }

  fn on_event(
    &mut self,
    _tree: &mut Tree,
    event: Event,
    layout: Layout,
    cursor: Cursor,
    renderer: &R,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, M>,
    viewport: &Rectangle,
  ) -> Status {
    // TODO: will event propagation actually do anything because currently "virtual" widgets have no state?

    let absolute_position = layout.position();
    let cursor_position = cursor.position();
    match &event {
      Event::Mouse(_) => {
        if let Some(cursor_position) = cursor_position {
          let mouse_position_relative = Point::new(cursor_position.x - absolute_position.x, cursor_position.y - absolute_position.y);
          if self.propagate_event_to_element_at(&event, mouse_position_relative, layout, cursor, renderer, clipboard, shell, viewport) == Status::Captured {
            return Status::Captured;
          }
        }
      }
      Event::Touch(touch_event) => {
        let touch_position_absolute = match touch_event {
          touch::Event::FingerPressed { position, .. } => position,
          touch::Event::FingerMoved { position, .. } => position,
          touch::Event::FingerLifted { position, .. } => position,
          touch::Event::FingerLost { position, .. } => position,
        };
        let touch_position_relative = Point::new(touch_position_absolute.x - absolute_position.x, touch_position_absolute.y - absolute_position.y);
        if self.propagate_event_to_element_at(&event, touch_position_relative, layout, cursor, renderer, clipboard, shell, viewport) == Status::Captured {
          return Status::Captured;
        }
      }
      // TODO: propagate other events?
      _ => {},
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

  fn draw(
    &self,
    _tree: &Tree,
    renderer: &mut R,
    theme: &R::Theme,
    style: &renderer::Style,
    layout: Layout,
    cursor: Cursor,
    viewport: &Rectangle,
  ) {
    // TODO: cache trees in (hash)map with (row_index, column_index) as index. Update this every time we draw based on viewport!

    let mut tree_cache = self.tree_cache.borrow_mut();
    tree_cache.clear();

    let absolute_position = layout.position();
    if self.num_rows == 0 {
      return;
    }

    let last_row_index = self.num_rows.saturating_sub(1);
    let row_height_plus_spacing = self.row_height + self.spacing;
    let start_offset = (((viewport.y - absolute_position.y) / row_height_plus_spacing).floor() as usize).min(last_row_index);
    // NOTE: + 1 on next line to ensure that last partially visible row is not culled.
    let num_rows_to_render = ((viewport.height / row_height_plus_spacing).ceil() as usize + 1).min(self.num_rows);

    let num_columns = self.width_fill_portions.len();
    let mut y_offset = absolute_position.y + start_offset as f32 * row_height_plus_spacing;
    // TODO: confirm this is looping over the right row index!
    for row_index in start_offset..start_offset + num_rows_to_render {
      // TODO: confirm this is looping over the right column index!
      for (column_index, base_layout) in (0..num_columns).into_iter().zip(layout.children()) {
        let element = (self.mapper)(row_index, column_index);
        // HACK: reconstruct layout of element to fix its y position based on `y_offset`.
        // HACK: construct new tree from widget and pass it down.
        // TODO: passing in a new Tree every time causes widgets inside the table to not keep any state!
        let mut tree = Tree::new(&element);
        let node = reconstruct_layout_node(base_layout.bounds(), y_offset, &element, &mut tree, renderer);
        let layout = Layout::new(&node);
        element.as_widget().draw(&tree, renderer, theme, style, layout, cursor, viewport);
        tree_cache.push(tree);
      }
      y_offset += self.row_height;
      if row_index < last_row_index { // Don't add spacing after last row.
        y_offset += self.spacing;
      }
    }
  }
}

impl<'a, F, M, R: Renderer> TableRows<F> where
  F: Fn(usize, usize) -> Element<'a, M, R> + 'a
{
  fn get_row_index_at(&self, y: f32) -> Option<usize> {
    // TODO: return None when row index > num_rows!
    if y < 0f32 { return None; } // Out of bounds
    let spacing = self.spacing;
    let row_height = self.row_height;
    let row_height_plus_spacing = row_height + spacing;
    let row_offset = (y / row_height_plus_spacing).ceil() as usize;
    let row_offset_without_spacing = (row_offset as f32 * row_height_plus_spacing) - spacing;
    if y > row_offset_without_spacing {
      None // On row spacing
    } else {
      Some(row_offset.saturating_sub(1)) // NOTE: + 1 because row_offset is 1-based. Why is this the case?
    }
  }

  fn get_column_index_and_layout_at<'l>(&self, x: f32, layout: &Layout<'l>) -> Option<(usize, Layout<'l>)> {
    let spacing = self.spacing;
    let mut offset = 0f32;
    for (column_index, column_layout) in layout.children().enumerate() {
      if x < offset { return None; } // On column spacing or out of bounds
      offset += column_layout.bounds().width;
      if x <= offset { return Some((column_index, column_layout)); }
      offset += spacing;
    }
    None
  }

  fn propagate_event_to_element_at(
    &mut self,
    event: &Event,
    point: Point,
    layout: Layout,
    cursor: Cursor,
    renderer: &R,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, M>,
    viewport: &Rectangle,
  ) -> Status {
    let absolute_position = layout.position();
    let row_height_plus_spacing = self.row_height + self.spacing;
    let column_index_and_layout = self.get_column_index_and_layout_at(point.x, &layout);
    let row_index = self.get_row_index_at(point.y);
    if let (Some((column_index, base_layout)), Some(row_index)) = (column_index_and_layout, row_index) {
      let mut element = (self.mapper)(row_index, column_index);
      let y_offset = absolute_position.y + row_index as f32 * row_height_plus_spacing;
      // HACK: reconstruct layout of element to fix its y position based on `y_offset`.
      // HACK: construct new tree from widget and pass it down.
      // TODO: passing in a new Tree every time causes widgets inside the table to not keep any state!
      let mut tree = Tree::new(&element);
      let node = reconstruct_layout_node(base_layout.bounds(), y_offset, &element, &mut tree, renderer);
      let layout = Layout::new(&node);
      element.as_widget_mut().on_event(&mut tree, event.clone(), layout, cursor, renderer, clipboard, shell, viewport)
    } else {
      Status::Ignored
    }
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
  elements: &[Element<'_, M, R>],
  trees: &mut [Tree],
  renderer: &R,
) -> Vec<Node> {
  let num_columns = width_fill_portions.len();
  let last_column_index = num_columns.saturating_sub(1);
  let num_spacers = last_column_index as f32;
  let total_spacing = spacing * num_spacers;
  let available_width = max_width - total_spacing;
  let total_fill_portion = width_fill_portions.iter().sum::<u32>() as f32; // TODO: cache
  let mut layouts = Vec::new();
  let mut x_offset = 0f32;
  for (i, ((width_fill_portion, element), tree)) in width_fill_portions.iter().zip(elements).zip(trees).enumerate() {
    let width = (*width_fill_portion as f32 / total_fill_portion) * available_width;
    let limits = Limits::new(Size::ZERO, Size::new(width, row_height));
    let mut layout = element.as_widget().layout(tree, renderer, &limits);
    layout.move_to(Point::new(x_offset, 0f32));
    layouts.push(layout);
    x_offset += width;
    if i < last_column_index { // Don't add spacing after last column.
      x_offset += spacing;
    }
  }
  layouts
}

// TODO: reduce code duplication with above function
fn layout_virtual_columns(
  max_width: f32,
  row_height: f32,
  spacing: f32,
  width_fill_portions: &[u32],
) -> Vec<Node> {
  let num_columns = width_fill_portions.len();
  let last_column_index = num_columns.saturating_sub(1);
  let num_spacers = last_column_index as f32;
  let total_spacing = spacing * num_spacers;
  let available_width = max_width - total_spacing;
  let total_fill_portion = width_fill_portions.iter().sum::<u32>() as f32; // TODO: cache
  let mut layouts = Vec::new();
  let mut x_offset = 0f32;
  for (i, width_fill_portion) in width_fill_portions.iter().enumerate() {
    let width = (*width_fill_portion as f32 / total_fill_portion) * available_width;
    let mut layout = Node::new(Size::new(width, row_height));
    layout.move_to(Point::new(x_offset, 0f32));
    layouts.push(layout);
    x_offset += width;
    if i < last_column_index { // Don't add spacing after last column.
      x_offset += spacing;
    }
  }
  layouts
}

fn reconstruct_layout_node<M, R: Renderer>(
  bounds: Rectangle,
  y_offset: f32,
  element: &Element<'_, M, R>,
  tree: &mut Tree,
  renderer: &R,
) -> Node {
  // HACK: Reconstruct the layout from `base_layout` which has a correct x position, but an incorrect y position
  //       which always points to the first row. This is needed so that we do not have to lay out all the cells of
  //       the table each time the layout changes, because that is slow for larger tables.
  let limits = Limits::new(Size::ZERO, Size::new(bounds.width, bounds.height));
  let mut node = element.as_widget().layout(tree, renderer, &limits);
  node.move_to(Point::new(bounds.x, y_offset));
  node
}


//
// Common child widget handling
//

fn on_event_to_children<'a, M, R: Renderer>(
  children: &mut [Element<'a, M, R>],
  tree: &mut Tree,
  event: Event,
  layout: Layout,
  cursor: Cursor,
  renderer: &R,
  clipboard: &mut dyn Clipboard,
  shell: &mut Shell<'_, M>,
  viewport: &Rectangle,
) -> Status {
  children.iter_mut()
    .zip(&mut tree.children)
    .zip(layout.children())
    .map(|((child, tree), layout)| {
      child.as_widget_mut().on_event(
        tree,
        event.clone(),
        layout,
        cursor,
        renderer,
        clipboard,
        shell,
        viewport
      )
    })
    .fold(Status::Ignored, Status::merge)
}
fn mouse_interaction_to_children<'a, M, R: Renderer>(
  children: &[Element<'a, M, R>],
  tree: &Tree,
  layout: Layout,
  cursor: Cursor,
  viewport: &Rectangle,
  renderer: &R
) -> Interaction {
  children.iter()
    .zip(&tree.children)
    .zip(layout.children())
    .map(|((child, tree), layout)| {
      child.as_widget().mouse_interaction(tree, layout, cursor, viewport, renderer)
    })
    .max()
    .unwrap_or_default()
}
fn operate_to_children<'a, M, R: Renderer>(
  children: &[Element<'a, M, R>],
  tree: &mut Tree,
  layout: Layout,
  renderer: &R,
  operation: &mut dyn Operation<M>
) {
  operation.container(None, layout.bounds(), &mut |operation| {
    children.iter()
      .zip(&mut tree.children)
      .zip(layout.children())
      .for_each(|((child, tree), layout)| {
        child.as_widget().operate(tree, layout, renderer, operation);
      });
  });
}
fn draw_children<'a, M, R: Renderer>(
  children: &[Element<'a, M, R>],
  tree: &Tree,
  renderer: &mut R,
  theme: &R::Theme,
  style: &renderer::Style,
  layout: Layout,
  cursor: Cursor,
  viewport: &Rectangle,
) {
  children.iter()
    .zip(&tree.children)
    .zip(layout.children())
    .for_each(|((child, tree), layout)| {
      child.as_widget().draw(tree, renderer, theme, style, layout, cursor, viewport);
    });
}
