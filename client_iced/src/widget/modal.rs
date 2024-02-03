use std::rc::Rc;

use iced::{Background, Border, Color, Element, Event, keyboard, Length, Rectangle, Size, Theme, Vector};
use iced::advanced::{Clipboard, Renderer, Shell};
use iced::advanced::graphics::core::touch;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Style};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::alignment::{Horizontal, Vertical};
use iced::event;
use iced::keyboard::key::Named;
use iced::mouse::{self, Cursor};
use iced::widget::container;

/// A widget that overlays an element over an underlay element in a modal way, disabling the underlay element.
pub struct Modal<'a, M, T, R, S> {
  overlay: Element<'a, M, T, R>,
  underlay: Element<'a, M, T, R>,

  on_press_underlay_area: Option<Rc<dyn Fn() -> M>>,
  on_esc_pressed: Option<Rc<dyn Fn() -> M>>,

  draw_over_underlay_only: bool,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,

  style: S,
}
impl<'a, M, T, R, S> Modal<'a, M, T, R, S> where
  M: 'a,
  R: Renderer + 'a,
  S: ModalStyle<Theme=T>
{
  /// Creates a new [`Modal`] that overlays `overlay` over `underlay`.
  pub fn new(
    overlay: impl Into<Element<'a, M, T, R>>,
    underlay: impl Into<Element<'a, M, T, R>>,
  ) -> Self {
    Self {
      overlay: overlay.into(),
      underlay: underlay.into(),

      on_press_underlay_area: None,
      on_esc_pressed: None,

      draw_over_underlay_only: false,
      horizontal_alignment: Horizontal::Center,
      vertical_alignment: Vertical::Center,

      style: S::default(),
    }
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

  /// Sets whether the modal background should be drawn over the underlay only (`true`), or whether it should be drawn
  /// over everything (`false`, the default).
  pub fn draw_over_underlay_only(mut self, draw_over_underlay_only: bool) -> Self {
    self.draw_over_underlay_only = draw_over_underlay_only;
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

  /// Sets the `style` of this modal.
  pub fn style(mut self, style: S) -> Self {
    self.style = style;
    self
  }
}
impl<'a, M, R> Modal<'a, M, Theme, R, ModalStyleForTheme> where
  M: 'a,
  R: Renderer + 'a,
{
  /// Creates a new [`Modal`] that wraps `overlay` in a modal-styled container, and overlaps it over `underlay`.
  pub fn with_container(
    overlay: impl Into<Element<'a, M, Theme, R>>,
    underlay: impl Into<Element<'a, M, Theme, R>>,
  ) -> Self {
    let overlay = container::Container::new(overlay)
      .padding(10)
      .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let background = palette.background.base;
        container::Appearance {
          text_color: Some(background.text),
          background: Some(background.color.into()),
          border: Border {
            radius: 10.0.into(),
            width: 2.0,
            color: palette.primary.weak.color,
          },
          ..container::Appearance::default()
        }
      });
    Self::new(overlay, underlay)
  }
}

/// The appearance of a modal.
#[derive(Clone, Copy, Debug)]
pub struct ModalAppearance {
  /// The [`Background`] between the underlay and overlay.
  pub background: Background,
}

/// A style that dictates the appearance of a modal.
pub trait ModalStyle: Default + Clone {
  /// The supported theme.
  type Theme;
  /// Produces the [`ModalBackground`] of a [`Modal`].
  fn appearance(&self, theme: &Self::Theme) -> ModalAppearance;
}

/// The style for a modal widget using the [built-in theme](Theme).
#[derive(Default, Clone)]
pub enum ModalStyleForTheme {
  /// Default style.
  #[default]
  Default,
  /// A custom style.
  Custom(Rc<dyn Fn(&Theme) -> ModalAppearance>),
}
impl ModalStyle for ModalStyleForTheme {
  type Theme = Theme;
  fn appearance(&self, theme: &Self::Theme) -> ModalAppearance {
    match self {
      Self::Default => {
        let mut background_base_color = theme.extended_palette().background.strong.color.inverse();
        background_base_color.a *= 0.75;
        let background = Background::Color(background_base_color);
        ModalAppearance { background }
      }
      Self::Custom(f) => f(theme),
    }
  }
}


/// Conversion into [`Element`].
impl<'a, M, T, R, S> From<Modal<'a, M, T, R, S>> for Element<'a, M, T, R> where
  M: 'a,
  R: Renderer + 'a,
  T: 'a,
  S: ModalStyle<Theme=T> + 'a,
{
  fn from(modal: Modal<'a, M, T, R, S>) -> Self {
    Self::new(modal)
  }
}


// Widget implementation
impl<M, T, R, S> Widget<M, T, R> for Modal<'_, M, T, R, S> where
  R: Renderer,
  S: ModalStyle<Theme=T>,
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

  fn size(&self) -> Size<Length> {
    self.underlay.as_widget().size()
  }
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
    translation: Vector,
  ) -> Option<overlay::Element<'o, M, T, R>> {
    let modal_overlay = ModalOverlay {
      underlay_bounds: self.draw_over_underlay_only.then(|| layout.bounds() + translation),
      horizontal_alignment: self.horizontal_alignment,
      vertical_alignment: self.vertical_alignment,
      overlay: &mut self.overlay,
      overlay_tree: &mut tree.children[1],

      on_press_underlay_area: self.on_press_underlay_area.clone(),
      on_esc_pressed: self.on_esc_pressed.clone(),

      style: self.style.clone(),
    };
    Some(overlay::Element::new(Box::new(modal_overlay)))
  }

  // Note: did not override `on_event`, `mouse_interaction`, and `operate` as the modal overlay disables the underlay.

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut R,
    theme: &T,
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
struct ModalOverlay<'a, 'o, M, T, R, S> {
  underlay_bounds: Option<Rectangle>,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,
  overlay: &'o mut Element<'a, M, T, R>,
  overlay_tree: &'o mut Tree,

  on_press_underlay_area: Option<Rc<dyn Fn() -> M>>,
  on_esc_pressed: Option<Rc<dyn Fn() -> M>>,

  style: S,
}
impl<M, T, R, S> overlay::Overlay<M, T, R> for ModalOverlay<'_, '_, M, T, R, S> where
  R: Renderer,
  S: ModalStyle<Theme=T>,
{
  fn layout(
    &mut self,
    renderer: &R,
    bounds: Size,
  ) -> Node {
    let limits = Limits::new(Size::ZERO, self.underlay_bounds.map_or(bounds, |b| b.size()));
    let max_size = limits.max();
    let overlay_node = self.overlay.as_widget()
      .layout(self.overlay_tree, renderer, &limits)
      .align(
        self.horizontal_alignment.into(),
        self.vertical_alignment.into(),
        max_size,
      );
    let node = Node::with_children(max_size, vec![overlay_node]);
    if let Some(underlay_bounds) = self.underlay_bounds {
      node.move_to(underlay_bounds.position())
    } else {
      node
    }
  }
  fn overlay(
    &mut self,
    layout: Layout<'_>,
    renderer: &R,
  ) -> Option<overlay::Element<M, T, R>> {
    let overlay_layout = layout.children().next().unwrap();
    self.overlay.as_widget_mut().overlay(
      self.overlay_tree,
      overlay_layout,
      renderer,
      Vector::ZERO, // TODO: do we need to pass in a different translation here?
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
      if let Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(Named::Escape), .. }) = event {
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
    theme: &T,
    style: &Style,
    layout: Layout<'_>,
    cursor: Cursor,
  ) {
    let bounds = layout.bounds();
    let appearance = self.style.appearance(theme);
    renderer.fill_quad(
      renderer::Quad {
        bounds,
        border: Border {
          radius: 0.0f32.into(),
          width: 0.0,
          color: Color::TRANSPARENT,
        },
        ..renderer::Quad::default()
      },
      appearance.background,
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
