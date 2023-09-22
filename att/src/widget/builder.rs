use std::borrow::Cow;

use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Row, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};

#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<E>(E);
impl<'a, M, R> Default for WidgetBuilder<internal::Empty<'a, M, R>> {
  fn default() -> Self { Self(Default::default()) }
}

// Builder methods for creating standalone widgets.
impl<E> WidgetBuilder<E> {
  pub fn space(self) -> SpaceBuilder<E> {
    SpaceBuilder::new(self.0)
  }
  pub fn text<'a>(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<E, Cow<'a, str>> {
    TextBuilder::new(self.0, content.into())
  }
  pub fn button<'a, R>(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<E, Element<'a, (), R>> {
    ButtonBuilder::new(self.0, content.into())
  }
}
impl<'a, E: internal::Push<'a>> WidgetBuilder<E> {
  pub fn element(self, element: impl Into<Element<'a, E::Message, E::Renderer>>) -> E::Builder {
    self.0.push(element.into())
  }
}
// Builder methods for creating container widgets with children widgets, such as rows and columns.
impl<'a, E: internal::Consume<'a>> WidgetBuilder<E> {
  pub fn into_row(self) -> RowBuilder<E> {
    RowBuilder::new(self.0)
  }
}
// Builder methods for taking the result of building.
impl<'a, E: internal::Take<'a>> WidgetBuilder<E> {
  pub fn take(self) -> E::Element {
    self.0.take()
  }
}

/// Builder for a [`Space`] widget.
#[must_use]
pub struct SpaceBuilder<E> {
  elements: E,
  width: Length,
  height: Length,
}
impl<E> SpaceBuilder<E> {
  fn new(elements: E) -> Self {
    Self {
      elements,
      width: Length::Shrink,
      height: Length::Shrink,
    }
  }
}
impl<E> SpaceBuilder<E> {
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }
}
impl<'a, E: internal::Push<'a>> SpaceBuilder<E> {
  pub fn done(self) -> E::Builder {
    self.elements.push(Space::new(self.width, self.height))
  }
}

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<E, C> {
  elements: E,
  content: C,
  size: Option<Pixels>,
}
impl<E, C> TextBuilder<E, C> {
  fn new(elements: E, content: C) -> Self {
    Self { elements, content, size: None }
  }

  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }
}
impl<'a, E: internal::Push<'a>> TextBuilder<E, Cow<'a, str>> where
  E::Renderer: TextRenderer,
  <E::Renderer as Renderer>::Theme: TextStyleSheet,
{
  pub fn done(self) -> E::Builder {
    let mut text = Text::new(self.content);
    if let Some(size) = self.size {
      text = text.size(size);
    }
    self.elements.push(text)
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<E, C> {
  elements: E,
  contents: C,
  disabled: bool,
}
impl<E, C> ButtonBuilder<E, C> {
  fn new(elements: E, contents: C) -> Self {
    Self { elements, contents, disabled: false }
  }

  pub fn enabled(mut self) -> Self {
    self.disabled = false;
    self
  }
  pub fn disabled(mut self) -> Self {
    self.disabled = true;
    self
  }
}
impl<'a, E: internal::Push<'a>> ButtonBuilder<E, Element<'a, (), E::Renderer>> where
  <E::Renderer as Renderer>::Theme: ButtonStyleSheet,
{
  pub fn done(self, on_press: impl Fn() -> E::Message + 'a) -> E::Builder {
    let mut button = Button::new(self.contents);
    if !self.disabled {
      button = button.on_press(());
    }
    let element = Element::new(button).map(move |_| on_press());
    self.elements.push(element)
  }
}

/// Builder for a [`Row`] widget.
#[must_use]
pub struct RowBuilder<E> {
  elements: E,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
}
impl<'a, E> RowBuilder<E> {
  fn new(elements: E) -> Self {
    Self {
      elements,
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      align_items: Alignment::Start,
    }
  }

  /// Sets the horizontal spacing _between_ elements.
  ///
  /// Custom margins per element do not exist in iced. You should use this
  /// method instead! While less flexible, it helps you keep spacing between
  /// elements consistent.
  pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
    self.spacing = spacing.into().0;
    self
  }
  /// Sets the [`Padding`] of the [`Row`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }
  /// Sets the width of the [`Row`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the height of the [`Row`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the vertical alignment of the contents of the [`Row`] .
  pub fn align_items(mut self, align: Alignment) -> Self {
    self.align_items = align;
    self
  }
}
impl<'a, E: internal::Consume<'a>> RowBuilder<E> {
  pub fn done(self) -> E::Builder {
    self.elements.consume(|vec| {
      Row::with_children(vec)
        .spacing(self.spacing)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .align_items(self.align_items)
        .into()
    })
  }
}

/// Internal state management for widget builder.
mod internal {
  use std::marker::PhantomData;

  use iced::advanced::Renderer;
  use iced::Element;

  use super::WidgetBuilder;

  /// Empty: 0 elements.
  #[repr(transparent)]
  pub struct Empty<'a, M, R>(PhantomData<&'a M>, PhantomData<R>);
  impl<'a, M, R> Default for Empty<'a, M, R> {
    fn default() -> Self { Self(PhantomData::default(), PhantomData::default()) }
  }
  /// 1 element.
  #[repr(transparent)]
  pub struct One<'a, M, R>(Element<'a, M, R>);
  /// >1 elements.
  #[repr(transparent)]
  pub struct Many<'a, M, R>(Vec<Element<'a, M, R>>);

  /// Internal trait for pushing elements onto a builder.
  pub trait Push<'a> {
    /// [`Element`] message type.
    type Message: 'a;
    /// [`Element`] renderer type.
    type Renderer: Renderer + 'a;
    /// Builder produced by [`push`].
    type Builder;
    /// Push the [`Element`] produced by `into_element` onto `self`, then return a new [builder](Self::Builder) with
    /// those elements.
    fn push<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Push<'a> for Empty<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn push<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      WidgetBuilder(One(element))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Push<'a> for One<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Builder = WidgetBuilder<Many<'a, M, R>>;
    fn push<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      let elements = vec![self.0, element];
      WidgetBuilder(Many(elements))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Push<'a> for Many<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Builder = WidgetBuilder<Many<'a, M, R>>;
    fn push<I: Into<Element<'a, Self::Message, Self::Renderer>>>(mut self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      self.0.push(element);
      WidgetBuilder(self)
    }
  }

  /// Internal trait for consuming elements from a builder.
  pub trait Consume<'a> {
    /// [`Element`] message type.
    type Message: 'a;
    /// [`Element`] renderer type.
    type Renderer: Renderer + 'a;
    /// Builder produced by [`consume`].
    type Builder;
    /// Consume the [elements](Element) from `self` into a [`Vec`], call `produce` on that [`Vec`] to create a new
    /// [`Element`], then return a new [builder](Self::Builder) with that element.
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Consume<'a> for One<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder {
      let elements = vec![self.0];
      let new_element = produce(elements);
      WidgetBuilder(One(new_element))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Consume<'a> for Many<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder {
      let elements = self.0;
      let new_element = produce(elements);
      WidgetBuilder(One(new_element))
    }
  }

  /// Internal trait for taking the single element from a builder.
  pub trait Take<'a> {
    /// [`Element`] type
    type Element;
    /// Take the single [`Element`] from `self` and return it.
    fn take(self) -> Self::Element;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Take<'a> for One<'a, M, R> {
    type Element = Element<'a, M, R>;
    fn take(self) -> Self::Element {
      self.0
    }
  }
}
