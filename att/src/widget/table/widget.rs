use iced::{Element, Event, Length, Point, Rectangle, Size};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, tree, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

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
impl<'a, M, R> Table<'a, M, R> {
  pub fn new(
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    spacing: f32,
    header: Element<'a, M, R>,
    rows: Element<'a, M, R>
  ) -> Self {
    Self { width, height, max_width, max_height, spacing, header, rows }
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

  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout<'_>, renderer: &R) -> Option<overlay::Element<'o, M, R>> {
    let (header_layout, rows_layout) = Self::unfold_layout(layout);
    let (header_tree, rows_tree, ) = Self::unfold_tree_mut(tree);
    if let Some(header_overlay) = self.header.as_widget_mut().overlay(header_tree, header_layout, renderer) {
      return Some(header_overlay)
    }
    self.rows.as_widget_mut().overlay(rows_tree, rows_layout, renderer)
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

impl<'a, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for Table<'a, M, R> {
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}
