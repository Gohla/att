//! Propagate [`iced::advanced::widget::Widget`] functions to child elements.

use iced::{Event, Rectangle};
use iced::advanced::{Clipboard, Layout, overlay, Renderer, renderer, Shell};
use iced::advanced::graphics::core::Element;
use iced::advanced::widget::{Operation, Tree};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

/// Propagate [`iced::advanced::widget::Widget::draw`] to child elements.
pub fn draw<'a, M, T, R: Renderer>(
  child_elements: &[Element<'a, M, T, R>],
  tree: &Tree,
  renderer: &mut R,
  theme: &T,
  style: &renderer::Style,
  layout: Layout,
  cursor: Cursor,
  viewport: &Rectangle,
) {
  child_elements.iter()
    .zip(&tree.children)
    .zip(layout.children())
    .for_each(|((child, tree), layout)| {
      child.as_widget().draw(tree, renderer, theme, style, layout, cursor, viewport);
    });
}

/// Propagate [`iced::advanced::widget::Widget::on_event`] to child elements.
pub fn on_event<'a, M, T, R: Renderer>(
  child_elements: &mut [Element<'a, M, T, R>],
  tree: &mut Tree,
  event: Event,
  layout: Layout,
  cursor: Cursor,
  renderer: &R,
  clipboard: &mut dyn Clipboard,
  shell: &mut Shell<'_, M>,
  viewport: &Rectangle,
) -> Status {
  child_elements.iter_mut()
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

/// Propagate [`iced::advanced::widget::Widget::mouse_interaction`] to child elements.
pub fn mouse_interaction<'a, M, T, R: Renderer>(
  child_elements: &[Element<'a, M, T, R>],
  tree: &Tree,
  layout: Layout,
  cursor: Cursor,
  viewport: &Rectangle,
  renderer: &R
) -> Interaction {
  child_elements.iter()
    .zip(&tree.children)
    .zip(layout.children())
    .map(|((child, tree), layout)| {
      child.as_widget().mouse_interaction(tree, layout, cursor, viewport, renderer)
    })
    .max()
    .unwrap_or_default()
}

/// Propagate [`iced::advanced::widget::Widget::operate`] to child elements.
pub fn operate<'a, M, T, R: Renderer>(
  child_elements: &[Element<'a, M, T, R>],
  tree: &mut Tree,
  layout: Layout,
  renderer: &R,
  operation: &mut dyn Operation<M>
) {
  operation.container(None, layout.bounds(), &mut |operation| {
    child_elements.iter()
      .zip(&mut tree.children)
      .zip(layout.children())
      .for_each(|((child, tree), layout)| {
        child.as_widget().operate(tree, layout, renderer, operation);
      });
  });
}

/// Propagate [`iced::advanced::widget::Widget::overlay`] to child elements.
pub fn overlay<'a, M, T, R: Renderer>(
  child_elements: &'a mut [Element<'_, M, T, R>],
  tree: &'a mut Tree,
  layout: Layout,
  renderer: &R,
) -> Option<overlay::Element<'a, M, T, R>> {
  overlay::from_children(child_elements, tree, layout, renderer)
}
