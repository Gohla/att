use iced::{Element, Event, Length, Rectangle, Size};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, tree, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

use crate::widget::table::layout_columns;

pub struct TableHeader<'a, M, R> {
  pub spacing: f32,
  pub row_height: f32,
  width_fill_portions: Vec<u32>,
  headers: Vec<Element<'a, M, R>>,
}
impl<'a, M, R> TableHeader<'a, M, R> {
  pub fn new(spacing: f32, row_height: f32) -> Self {
    Self { spacing, row_height, width_fill_portions: Vec::new(), headers: Vec::new() }
  }

  pub fn push_column(&mut self, width_fill_portion: u32, header: impl Into<Element<'a, M, R>>) {
    self.width_fill_portions.push(width_fill_portion);
    self.headers.push(header.into());
  }
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
    let layouts = layout_columns(total_width, self.row_height, self.spacing, &self.width_fill_portions, Some((&self.headers, &mut tree.children, renderer)));
    Node::with_children(Size::new(total_width, self.row_height), layouts)
  }
  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout, renderer: &R) -> Option<overlay::Element<'o, M, R>> {
    crate::widget::child::overlay(&mut self.headers, tree, layout, renderer)
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
    crate::widget::child::draw(&self.headers, tree, renderer, theme, style, layout, cursor, viewport)
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
    crate::widget::child::on_event(&mut self.headers, tree, event, layout, cursor, renderer, clipboard, shell, viewport)
  }
  fn mouse_interaction(&self, tree: &Tree, layout: Layout, cursor: Cursor, viewport: &Rectangle, renderer: &R) -> Interaction {
    crate::widget::child::mouse_interaction(&self.headers, tree, layout, cursor, viewport, renderer)
  }
  fn operate(&self, tree: &mut Tree, layout: Layout, renderer: &R, operation: &mut dyn Operation<M>) {
    crate::widget::child::operate(&self.headers, tree, layout, renderer, operation)
  }
}

impl<'a, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for TableHeader<'a, M, R> {
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}
