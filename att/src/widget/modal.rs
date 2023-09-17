use std::sync::Arc;

use iced::{Color, Element, Event, Length, Point, Rectangle, Size};
use iced::advanced::{self, Clipboard, Shell};
use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::alignment::Alignment;
use iced::event;
use iced::mouse::{self, Cursor};

/// A widget that centers a modal element over a parent element.
pub struct Modal<'a, M, R> {
  parent: Element<'a, M, R>,
  modal: Element<'a, M, R>,
  on_press_parent_area: Option<Arc<dyn Fn() -> M>>,
}

impl<'a, M, R> Modal<'a, M, R> {
  /// Creates a new [`Modal`] that centers the `modal` element over the `parent` element.
  pub fn new(
    parent: impl Into<Element<'a, M, R>>,
    modal: impl Into<Element<'a, M, R>>,
  ) -> Self {
    Self {
      parent: parent.into(),
      modal: modal.into(),
      on_press_parent_area: None,
    }
  }

  /// Sets the `message_producer` to call when the parent (background) area of this modal is pressed.
  pub fn on_press_parent_area(mut self, message_producer: impl Fn() -> M + 'static) -> Self {
    self.on_press_parent_area = Some(Arc::new(message_producer));
    self
  }
}

impl<'a, M, R: advanced::Renderer> Widget<M, R> for Modal<'a, M, R> {
  fn children(&self) -> Vec<Tree> {
    vec![
      Tree::new(&self.parent),
      Tree::new(&self.modal),
    ]
  }

  fn width(&self) -> Length {
    self.parent.as_widget().width()
  }

  fn height(&self) -> Length {
    self.parent.as_widget().height()
  }

  fn layout(
    &self,
    tree: &mut Tree,
    renderer: &R,
    limits: &layout::Limits,
  ) -> Node {
    self.parent.as_widget().layout(
      &mut tree.children[0],
      renderer,
      limits,
    )
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut R,
    theme: &<R as advanced::Renderer>::Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: Cursor,
    viewport: &Rectangle,
  ) {
    self.parent.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      style,
      layout,
      cursor,
      viewport,
    );
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&[&self.parent, &self.modal]);
  }

  fn operate(
    &self,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &R,
    operation: &mut dyn widget::Operation<M>,
  ) {
    self.parent.as_widget().operate(
      &mut tree.children[0],
      layout,
      renderer,
      operation,
    );
  }

  fn on_event(
    &mut self,
    tree: &mut Tree,
    event: Event,
    layout: Layout<'_>,
    cursor: Cursor,
    renderer: &R,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, M>,
    viewport: &Rectangle,
  ) -> event::Status {
    self.parent.as_widget_mut().on_event(
      &mut tree.children[0],
      event,
      layout,
      cursor,
      renderer,
      clipboard,
      shell,
      viewport,
    )
  }

  fn mouse_interaction(
    &self,
    state: &Tree,
    layout: Layout<'_>,
    cursor: Cursor,
    viewport: &Rectangle,
    renderer: &R,
  ) -> mouse::Interaction {
    self.parent.as_widget().mouse_interaction(
      &state.children[0],
      layout,
      cursor,
      viewport,
      renderer,
    )
  }

  fn overlay<'b>(
    &'b mut self,
    state: &'b mut Tree,
    layout: Layout<'_>,
    _renderer: &R,
  ) -> Option<overlay::Element<'b, M, R>> {
    let modal_overlay = ModalOverlay {
      content: &mut self.modal,
      tree: &mut state.children[1],
      size: layout.bounds().size(),
      on_press_parent_area: self.on_press_parent_area.clone(),
    };
    Some(overlay::Element::new(layout.position(), Box::new(modal_overlay)))
  }
}

struct ModalOverlay<'a, 'b, M, R> {
  content: &'b mut Element<'a, M, R>,
  tree: &'b mut Tree,
  size: Size,
  on_press_parent_area: Option<Arc<dyn Fn() -> M>>,
}

impl<'a, 'b, M, R: advanced::Renderer> overlay::Overlay<M, R> for ModalOverlay<'a, 'b, M, R> {
  fn layout(
    &mut self,
    renderer: &R,
    _bounds: Size,
    position: Point,
  ) -> Node {
    let limits = layout::Limits::new(Size::ZERO, self.size)
      .width(Length::Fill)
      .height(Length::Fill);

    let mut child = self
      .content
      .as_widget()
      .layout(self.tree, renderer, &limits);

    child.align(Alignment::Center, Alignment::Center, limits.max());

    let mut node = Node::with_children(self.size, vec![child]);
    node.move_to(position);

    node
  }

  fn draw(
    &self,
    renderer: &mut R,
    theme: &R::Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: Cursor,
  ) {
    renderer.fill_quad(
      renderer::Quad {
        bounds: layout.bounds(),
        border_radius: Default::default(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
      },
      Color {
        a: 0.80,
        ..Color::BLACK
      },
    );

    self.content.as_widget().draw(
      self.tree,
      renderer,
      theme,
      style,
      layout.children().next().unwrap(),
      cursor,
      &layout.bounds(),
    );
  }

  fn operate(
    &mut self,
    layout: Layout<'_>,
    renderer: &R,
    operation: &mut dyn widget::Operation<M>,
  ) {
    self.content.as_widget().operate(
      self.tree,
      layout.children().next().unwrap(),
      renderer,
      operation,
    );
  }

  fn on_event(
    &mut self,
    event: Event,
    layout: Layout<'_>,
    cursor: Cursor,
    renderer: &R,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, M>,
  ) -> event::Status {
    let content_bounds = layout.children().next().unwrap().bounds();

    if let Some(message_producer) = self.on_press_parent_area.as_ref() {
      if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = &event {
        if !cursor.is_over(content_bounds) {
          shell.publish(message_producer());
          return event::Status::Captured;
        }
      }
    }

    self.content.as_widget_mut().on_event(
      self.tree,
      event,
      layout.children().next().unwrap(),
      cursor,
      renderer,
      clipboard,
      shell,
      &layout.bounds(),
    )
  }

  fn mouse_interaction(
    &self,
    layout: Layout<'_>,
    cursor: Cursor,
    viewport: &Rectangle,
    renderer: &R,
  ) -> mouse::Interaction {
    self.content.as_widget().mouse_interaction(
      self.tree,
      layout.children().next().unwrap(),
      cursor,
      viewport,
      renderer,
    )
  }

  fn overlay<'c>(
    &'c mut self,
    layout: Layout<'_>,
    renderer: &R,
  ) -> Option<overlay::Element<'c, M, R>> {
    self.content.as_widget_mut().overlay(
      self.tree,
      layout.children().next().unwrap(),
      renderer,
    )
  }
}

impl<'a, M: 'a, R: advanced::Renderer + 'a> From<Modal<'a, M, R>> for Element<'a, M, R> {
  fn from(modal: Modal<'a, M, R>) -> Self {
    Self::new(modal)
  }
}
