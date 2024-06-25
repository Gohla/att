use iced::{Alignment, Element, Event, Length, Point, Rectangle, Size, Vector};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

/// A row where [constraints](Constraint) are applied to each element in the row.
pub struct ConstrainedRow<'a, M, T, R> {
  spacing: f32,
  height: f32,
  constraints: Vec<Constraint>,
  elements: Vec<Element<'a, M, T, R>>,
}

/// A constraint to apply to an element in a [constrained row](ConstrainedRow).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Constraint {
  width_fill_portion: f32,
  horizontal_alignment: Alignment,
  vertical_alignment: Alignment,
}
impl Default for Constraint {
  fn default() -> Self {
    Self {
      width_fill_portion: 1.0,
      horizontal_alignment: Alignment::Start,
      vertical_alignment: Alignment::Center
    }
  }
}
impl From<f32> for Constraint {
  fn from(width_fill_portion: f32) -> Self {
    Self { width_fill_portion, ..Self::default() }
  }
}
impl From<u32> for Constraint {
  fn from(width_fill_portion: u32) -> Self {
    Self::from(width_fill_portion as f32)
  }
}

impl<'a, M, T, R> ConstrainedRow<'a, M, T, R> {
  /// Creates a new constrained row without any constraints and elements. Consider using
  /// [with_constraints_and_elements](Self::with_constraints_and_elements) or [with_capacity](Self::with_capacity) to
  /// reduce [`Vec`] resize allocations.
  pub fn new() -> Self {
    Self::with_constraints_and_elements(Vec::new(), Vec::new())
  }
  /// Creates a new constrained row with `constraints` for widths and alignments of `elements`.
  ///
  /// If `constraints` is not the same size as `elements`, `constraints` will be resized to be the same size as
  /// `header_elements`, adding default constraints if needed.
  pub fn with_constraints_and_elements(
    mut constraints: Vec<Constraint>,
    elements: Vec<Element<'a, M, T, R>>,
  ) -> Self {
    constraints.resize_with(elements.len(), Default::default);
    Self {
      spacing: 1.0,
      height: 24.0,
      constraints,
      elements,
    }
  }
  /// Creates a new constrained row without any constraints and elements, but reserves `capacity` in the constraints and
  /// elements [`Vec`]s.
  pub fn with_capacity(capacity: usize) -> Self {
    Self::with_constraints_and_elements(Vec::with_capacity(capacity), Vec::with_capacity(capacity))
  }

  /// Sets the horizontal `spacing` _between_ elements of the row.
  pub fn spacing(mut self, spacing: f32) -> Self {
    self.spacing = spacing;
    self
  }
  /// Sets the `height` of the row.
  pub fn height(mut self, height: f32) -> Self {
    self.height = height;
    self
  }

  /// Appends `constraint` and `element` to the constraints and elements of the row.
  pub fn push(mut self, constraint: impl Into<Constraint>, element: impl Into<Element<'a, M, T, R>>) -> Self {
    self.constraints.push(constraint.into());
    self.elements.push(element.into());
    self
  }
}

impl<'a, M, T, R> Into<Element<'a, M, T, R>> for ConstrainedRow<'a, M, T, R> where
  M: 'a,
  T: 'a,
  R: Renderer + 'a
{
  fn into(self) -> Element<'a, M, T, R> {
    Element::new(self)
  }
}

impl<'a, M, T, R: Renderer> Widget<M, T, R> for ConstrainedRow<'a, M, T, R> {
  fn children(&self) -> Vec<Tree> {
    self.elements.iter().map(Tree::new).collect()
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&self.elements);
  }

  fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fixed(self.height)) }
  fn layout(&self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
    let limits = limits.max_height(self.height);
    let max = limits.max();

    let cells = self.elements.len();
    let total_fill_portion: f32 = self.constraints.iter().map(|c| c.width_fill_portion).sum();
    let available_width = max.width - (self.spacing * cells.saturating_sub(1) as f32);

    let mut nodes = Vec::with_capacity(cells);
    let mut x = 0.0;
    for ((element, constraint), tree) in self.elements.iter().zip(&self.constraints).zip(&mut tree.children) {
      let width = (constraint.width_fill_portion / total_fill_portion) * available_width;
      let element_limits = limits.max_width(width);
      let node = element.as_widget()
        .layout(tree, renderer, &element_limits)
        .move_to(Point::new(x, 0.0))
        .align(constraint.horizontal_alignment, constraint.vertical_alignment, element_limits.max());
      nodes.push(node);
      x += width + self.spacing;
    }
    Node::with_children(max, nodes)
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut R,
    theme: &T,
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
  fn operate(&self, tree: &mut Tree, layout: Layout, renderer: &R, operation: &mut dyn Operation<()>) {
    crate::widget::child::operate(&self.elements, tree, layout, renderer, operation)
  }

  fn overlay<'o>(&'o mut self, tree: &'o mut Tree, layout: Layout, renderer: &R, translation: Vector) -> Option<overlay::Element<'o, M, T, R>> {
    crate::widget::child::overlay(&mut self.elements, tree, layout, renderer, translation)
  }
}
