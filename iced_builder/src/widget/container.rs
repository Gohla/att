use iced::{Length, Padding, Pixels};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Container, container};

use crate::internal::state::{Elem, StateMap};

/// Builder for a [`Container`] widget.
#[must_use]
pub struct ContainerBuilder<'a, S: StateMap> where
  S::Theme: container::Catalog
{
  state: S,
  id: Option<container::Id>,
  padding: Padding,
  width: Length,
  height: Length,
  max_width: f32,
  max_height: f32,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,
  class: <S::Theme as container::Catalog>::Class<'a>,
}
impl<'a, S: StateMap> ContainerBuilder<'a, S> where
  S::Theme: container::Catalog
{
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      id: None,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      max_width: f32::INFINITY,
      max_height: f32::INFINITY,
      horizontal_alignment: Horizontal::Left,
      vertical_alignment: Vertical::Top,
      class: <S::Theme as container::Catalog>::default(),
    }
  }


  /// Sets the [`container::Id`] of the [`Container`].
  pub fn id(mut self, id: container::Id) -> Self {
    self.id = Some(id);
    self
  }

  /// Sets the [`Padding`] of the [`Container`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }

  /// Sets the width of the [`Container`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  /// Sets the height of the [`Container`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  /// Sets the maximum width of the [`Container`].
  pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
    self.max_width = max_width.into().0;
    self
  }

  /// Sets the maximum height of the [`Container`].
  pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
    self.max_height = max_height.into().0;
    self
  }

  /// Sets the content alignment for the horizontal axis of the [`Container`].
  pub fn align_x(mut self, alignment: Horizontal) -> Self {
    self.horizontal_alignment = alignment;
    self
  }

  /// Sets the content alignment for the vertical axis of the [`Container`].
  pub fn align_y(mut self, alignment: Vertical) -> Self {
    self.vertical_alignment = alignment;
    self
  }

  /// Centers the contents in the horizontal axis of the [`Container`].
  pub fn center_x(mut self) -> Self {
    self.horizontal_alignment = Horizontal::Center;
    self
  }

  /// Centers the contents in the vertical axis of the [`Container`].
  pub fn center_y(mut self) -> Self {
    self.vertical_alignment = Vertical::Center;
    self
  }


  /// Sets the `styler` function of the [`Container`].
  pub fn style(mut self, styler: impl Fn(&S::Theme) -> container::Style + 'a) -> Self where
    <S::Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, S::Theme>>,
  {
    self.class = (Box::new(styler) as container::StyleFn<'a, S::Theme>).into();
    self
  }

  /// Sets the `class` of the [`Container`].
  pub fn class(mut self, class: impl Into<<S::Theme as container::Catalog>::Class<'a>>) -> Self {
    self.class = class.into();
    self
  }


  /// Takes the last element out of the builder, creates the [`Container`] with that element, then adds the container
  /// to the builder and returns the builder.
  pub fn add(self) -> S::MapOutput where
    S::Element: Into<Elem<'a, S>>, // For `Container::new`
    Container<'a, S::Message, S::Theme, S::Renderer>: Into<S::Element>, // For `container.into()`
  {
    self.state.map_last(|content| {
      let mut container = Container::new(content)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .max_width(self.max_width)
        .max_height(self.max_width)
        .align_x(self.horizontal_alignment)
        .align_y(self.vertical_alignment)
        .class(self.class)
        ;
      if let Some(id) = self.id {
        container = container.id(id);
      }
      container.into()
    })
  }
}
