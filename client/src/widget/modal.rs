use std::rc::Rc;

use iced::{Background, Color, Element, Event, keyboard, Length, Point, Rectangle, Size, Theme, Vector};
use iced::advanced::{Clipboard, Renderer, Shell};
use iced::advanced::graphics::core::touch;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Style};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::alignment::{Horizontal, Vertical};
use iced::event;
use iced::mouse::{self, Cursor};
use iced::widget::container;

/// A widget that overlays an element over an underlay element in a modal way, disabling the underlay element.
pub struct Modal<'a, M, R> {
  overlay: Element<'a, M, R>,
  underlay: Element<'a, M, R>,

  on_press_underlay_area: Option<Rc<dyn Fn() -> M>>,
  on_esc_pressed: Option<Rc<dyn Fn() -> M>>,

  background: ModalBackground,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,
}
impl<'a, M, R> Modal<'a, M, R> where
  M: 'a,
  R: Renderer<Theme=Theme> + 'a,
{
  /// Creates a new [`Modal`] that overlays `overlay` over `underlay`.
  pub fn new(
    overlay: impl Into<Element<'a, M, R>>,
    underlay: impl Into<Element<'a, M, R>>,
  ) -> Self {
    Self {
      overlay: overlay.into(),
      underlay: underlay.into(),

      on_press_underlay_area: None,
      on_esc_pressed: None,

      background: ModalBackground::default(),
      horizontal_alignment: Horizontal::Center,
      vertical_alignment: Vertical::Center,
    }
  }
  /// Creates a new [`Modal`] that wraps `overlay` in a modal-styled container, and overlaps it over `underlay`.
  pub fn with_container(
    overlay: impl Into<Element<'a, M, R>>,
    underlay: impl Into<Element<'a, M, R>>,
  ) -> Self {
    let overlay = container::Container::new(overlay)
      .padding(10)
      .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let background = palette.background.base;
        container::Appearance {
          text_color: Some(background.text),
          background: Some(background.color.into()),
          border_radius: 10.0.into(),
          border_width: 2.0,
          border_color: palette.primary.weak.color,
          ..container::Appearance::default()
        }
      });
    Self::new(overlay, underlay)
  }

  /// Sets the `message_producer` to call when the modal should be closed due to either:
  /// - the underlay (background) area of this modal being pressed,
  /// - the [escape key](keyboard::KeyCode::Escape) being pressed.
  /// This sets both [`on_press_underlay_area`] and [`on_esc_pressed`] to `message_producer`.
  pub fn on_close_modal(mut self, message_producer: impl Fn() -> M + 'static) -> Self {
    let message_producer = Rc::new(message_producer);
    self.on_press_underlay_area = Some(message_producer.clone());
    self.on_esc_pressed = Some(message_producer);
    self
  }
  /// Sets the `message_producer` to call when the underlay (background) area of this modal is pressed.
  pub fn on_press_underlay_area(mut self, message_producer: impl Fn() -> M + 'static) -> Self {
    self.on_press_underlay_area = Some(Rc::new(message_producer));
    self
  }
  /// Sets the `message_producer` to call when the [escape key](keyboard::KeyCode::Escape) is pressed.
  pub fn on_esc_pressed(mut self, message_producer: impl Fn() -> M + 'static) -> Self {
    self.on_esc_pressed = Some(Rc::new(message_producer));
    self
  }
  /// Sets the `background` of this modal.
  pub fn background(mut self, background: ModalBackground) -> Self {
    self.background = background;
    self
  }
  /// Sets the `horizontal_alignment` of this modal.
  pub fn horizontal_alignment(mut self, horizontal_alignment: Horizontal) -> Self {
    self.horizontal_alignment = horizontal_alignment;
    self
  }
  /// Sets the `horizontal_alignment` of this modal.
  pub fn vertical_alignment(mut self, vertical_alignment: Vertical) -> Self {
    self.vertical_alignment = vertical_alignment;
    self
  }
}

/// Background for a [`Modal`].
#[derive(Clone, Default)]
pub enum ModalBackground {
  #[default]
  Default,
  Custom(Background),
  CustomThemed(Rc<dyn Fn(&Theme) -> Background>),
}
impl ModalBackground {
  /// Custom `background`.
  pub fn custom(background: Background) -> Self {
    Self::Custom(background)
  }
  /// Custom background based on `background_fn` with has access to [`Theme`].
  pub fn custom_themed(background_fn: impl Fn(&Theme) -> Background + 'static) -> Self {
    Self::CustomThemed(Rc::new(background_fn))
  }
}

/// Conversion into [`Element`].
impl<'a, M, R> From<Modal<'a, M, R>> for Element<'a, M, R> where
  M: 'a,
  R: Renderer<Theme=Theme> + 'a,
{
  fn from(modal: Modal<'a, M, R>) -> Self {
    Self::new(modal)
  }
}


// Widget implementation
impl<M, R> Widget<M, R> for Modal<'_, M, R> where
  R: Renderer<Theme=Theme>,
{
  fn children(&self) -> Vec<Tree> {
    vec![
      Tree::new(&self.underlay),
      Tree::new(&self.overlay),
    ]
  }
  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&[&self.underlay, &self.overlay]);
  }

  fn width(&self) -> Length { self.underlay.as_widget().width() }
  fn height(&self) -> Length { self.underlay.as_widget().height() }
  fn layout(
    &self,
    tree: &mut Tree,
    renderer: &R,
    limits: &Limits,
  ) -> Node {
    self.underlay.as_widget().layout(
      &mut tree.children[0],
      renderer,
      limits,
    )
  }
  fn overlay<'o>(
    &'o mut self,
    tree: &'o mut Tree,
    layout: Layout<'_>,
    _renderer: &R,
  ) -> Option<overlay::Element<'o, M, R>> {
    let modal_overlay = ModalOverlay {
      overlay: &mut self.overlay,
      overlay_tree: &mut tree.children[1],
      on_press_underlay_area: self.on_press_underlay_area.clone(),
      on_esc_pressed: self.on_esc_pressed.clone(),
      background: self.background.clone(),
      horizontal_alignment: self.horizontal_alignment,
      vertical_alignment: self.vertical_alignment,
    };
    Some(overlay::Element::new(layout.position(), Box::new(modal_overlay)))
  }

  // Note: did not override `on_event`, `mouse_interaction`, and `operate` as the modal overlay disables the underlay.

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut R,
    theme: &R::Theme,
    style: &Style,
    layout: Layout<'_>,
    cursor: Cursor,
    viewport: &Rectangle,
  ) {
    self.underlay.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      style,
      layout,
      cursor,
      viewport,
    );
  }
}

// Overlay implementation
struct ModalOverlay<'a, 'o, M, R> {
  overlay: &'o mut Element<'a, M, R>,
  overlay_tree: &'o mut Tree,
  on_press_underlay_area: Option<Rc<dyn Fn() -> M>>,
  on_esc_pressed: Option<Rc<dyn Fn() -> M>>,
  background: ModalBackground,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,
}
impl<M, R> overlay::Overlay<M, R> for ModalOverlay<'_, '_, M, R> where
  R: Renderer<Theme=Theme>,
{
  fn layout(
    &mut self,
    renderer: &R,
    bounds: Size,
    _position: Point,
    _translation: Vector,
  ) -> Node {
    let limits = Limits::new(Size::ZERO, bounds);
    let mut overlay_node = self.overlay.as_widget().layout(self.overlay_tree, renderer, &limits);
    let max_size = limits.max();
    overlay_node.align(
      self.horizontal_alignment.into(),
      self.vertical_alignment.into(),
      max_size,
    );
    Node::with_children(max_size, vec![overlay_node])
  }
  fn overlay(
    &mut self,
    layout: Layout<'_>,
    renderer: &R,
  ) -> Option<overlay::Element<M, R>> {
    let overlay_layout = layout.children().next().unwrap();
    self.overlay.as_widget_mut().overlay(
      self.overlay_tree,
      overlay_layout,
      renderer,
    )
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
    let overlay_layout = layout.children().next().unwrap();

    if let Some(on_press_underlay_area) = self.on_press_underlay_area.as_ref() {
      let overlay_bounds = overlay_layout.bounds();
      let pressed_underlay_area = match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => !cursor.is_over(overlay_bounds),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => !overlay_bounds.contains(position),
        _ => false,
      };
      if pressed_underlay_area {
        shell.publish(on_press_underlay_area());
        return event::Status::Captured;
      }
    }

    if let Some(on_esc_pressed) = self.on_esc_pressed.as_ref() {
      if let Event::Keyboard(keyboard::Event::KeyPressed { key_code: keyboard::KeyCode::Escape, .. }) = event {
        shell.publish(on_esc_pressed());
        return event::Status::Captured;
      }
    }

    self.overlay.as_widget_mut().on_event(
      self.overlay_tree,
      event,
      overlay_layout,
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
    let overlay_layout = layout.children().next().unwrap();
    self.overlay.as_widget().mouse_interaction(
      self.overlay_tree,
      overlay_layout,
      cursor,
      viewport,
      renderer,
    )
  }
  fn operate(
    &mut self,
    layout: Layout<'_>,
    renderer: &R,
    operation: &mut dyn Operation<M>,
  ) {
    let overlay_layout = layout.children().next().unwrap();
    self.overlay.as_widget().operate(
      self.overlay_tree,
      overlay_layout,
      renderer,
      operation,
    );
  }

  fn draw(
    &self,
    renderer: &mut R,
    theme: &R::Theme,
    style: &Style,
    layout: Layout<'_>,
    cursor: Cursor,
  ) {
    let bounds = layout.bounds();

    renderer.fill_quad(
      renderer::Quad {
        bounds,
        border_radius: 0.0f32.into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
      },
      self.background.get_background_color(theme),
    );

    let overlay_layout = layout.children().next().unwrap();
    self.overlay.as_widget().draw(
      self.overlay_tree,
      renderer,
      theme,
      style,
      overlay_layout,
      cursor,
      &bounds,
    );
  }
}
impl ModalBackground {
  fn get_background_color(&self, theme: &Theme) -> Background {
    match self {
      ModalBackground::Default => {
        let mut background_base_color = theme.extended_palette().background.strong.color.inverse();
        background_base_color.a *= 0.75;
        Background::Color(background_base_color)
      },
      ModalBackground::Custom(background) => *background,
      ModalBackground::CustomThemed(background_fn) => background_fn(theme),
    }
  }
}
