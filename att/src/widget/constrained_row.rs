use iced::{Alignment, Element, Event, Length, Point, Rectangle};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

pub struct ConstrainedRow<'a, M, R> {
  pub spacing: f32,
  pub height: f32,
  elements: Vec<Element<'a, M, R>>,
  constraints: Vec<Constraint>,
}

pub struct Constraint {
  width_fill_portion: f32,
  horizontal_alignment: Alignment,
  vertical_alignment: Alignment,
}
impl Default for Constraint {
  fn default() -> Self {
    Self { width_fill_portion: 1.0, horizontal_alignment: Alignment::Start, vertical_alignment: Alignment::Start }
  }
}

impl<'a, M, R> ConstrainedRow<'a, M, R> {
  pub fn new() -> Self {
    Self::with_elements_and_constraints(Vec::new(), Vec::new())
  }
  pub fn with_elements_and_constraints(
    elements: Vec<Element<'a, M, R>>,
    mut constraints: Vec<Constraint>,
  ) -> Self {
    constraints.resize_with(elements.len(), Default::default);
    Self {
      spacing: 0.0,
      height: 26.0,
      elements,
      constraints,
    }
  }
  pub fn with_capacity(capacity: usize) -> Self {
    Self::with_elements_and_constraints(Vec::with_capacity(capacity), Vec::with_capacity(capacity))
  }

  pub fn push(mut self, element: impl Into<Element<'a, M, R>>, constraint: Constraint) -> Self {
    self.elements.push(element.into());
    self.constraints.push(constraint);
    self
  }
}

impl<'a, M, R: Renderer> Widget<M, R> for ConstrainedRow<'a, M, R> {
  fn state(&self) -> tree::State { tree::State::None }
  fn tag(&self) -> tree::Tag { tree::Tag::stateless() }
  fn children(&self) -> Vec<Tree> {
    self.elements.iter().map(Tree::new).collect()
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&self.elements);
  }

  fn width(&self) -> Length { Length::Fill }
  fn height(&self) -> Length { Length::Fixed(self.height) }
  fn layout(&self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
    let limits = limits.height(self.height);
    let max = limits.max();

    let cells = self.elements.len();
    let total_fill_portion: f32 = self.constraints.iter().map(|c| c.width_fill_portion).sum();
    let available_width = max.width - (self.spacing * cells.saturating_sub(1) as f32);

    let mut nodes = Vec::with_capacity(cells);
    let mut x = 0.0;
    for ((element, constraint), tree) in self.elements.iter().zip(&self.constraints).zip(&mut tree.children) {
      let width = (constraint.width_fill_portion / total_fill_portion) * available_width;
      let element_limits = limits.width(width);
      let mut node = element.as_widget().layout(tree, renderer, &element_limits);
      node.move_to(Point::new(x, 0.0));
      node.align(constraint.horizontal_alignment, constraint.vertical_alignment, element_limits.fill());
      nodes.push(node);
      x += width + self.spacing;
    }
    Node::with_children(max, nodes)
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
    crate::widget::child::draw(&self.elements, tree, renderer, theme, style, layout, cursor, viewport)
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
    crate::widget::child::on_event(&mut self.elements, tree, event, layout, cursor, renderer, clipboard, shell, viewport)
  }
  fn mouse_interaction(&self, tree: &Tree, layout: Layout, cursor: Cursor, viewport: &Rectangle, renderer: &R) -> Interaction {
    crate::widget::child::mouse_interaction(&self.elements, tree, layout, cursor, viewport, renderer)
  }
  fn operate(&self, tree: &mut Tree, layout: Layout, renderer: &R, operation: &mut dyn Operation<M>) {
    crate::widget::child::operate(&self.elements, tree, layout, renderer, operation)
  }

  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout, renderer: &R) -> Option<overlay::Element<'o, M, R>> {
    crate::widget::child::overlay(&mut self.elements, tree, layout, renderer)
  }
}

impl<'a, M: 'a, R: Renderer + 'a> Into<Element<'a, M, R>> for ConstrainedRow<'a, M, R> {
  fn into(self) -> Element<'a, M, R> {
    Element::new(self)
  }
}
