use std::borrow::Cow;

use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Row, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};

use internal::{Elements0, Elements1, ConsumeElements, Elements};

#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<E>(E);
impl<'a, M, R> Default for WidgetBuilder<Elements0<'a, M, R>> {
  fn default() -> Self { Self(Elements0::default()) }
}

// Builder methods for creating standalone widgets.
impl<E> WidgetBuilder<E> {
  pub fn space(self) -> SpaceBuilder<E> {
    SpaceBuilder::new(self.0)
  }
  pub fn text<'a>(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<Cow<'a, str>, E> {
    TextBuilder::new(content.into(), self.0)
  }
  pub fn button<'a, R>(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<Element<'a, (), R>, E> {
    ButtonBuilder::new(content.into(), self.0)
  }
}
impl<'a, E: Elements<'a>> WidgetBuilder<E> {
  pub fn element(self, element: impl Into<Element<'a, E::Message, E::Renderer>>) -> E::Output {
    self.0.push(element.into())
  }
}
// Builder methods for creating container widgets that have multiple children widgets, such as [`Row`] and [`Col`].
impl<'a, E: ConsumeElements<'a>> WidgetBuilder<E> {
  pub fn into_row(self) -> RowBuilder<E> {
    RowBuilder::new(self.0)
  }
}
// Builder methods for taking the result of building.
impl<'a, M, R> WidgetBuilder<Elements1<'a, M, R>> {
  pub fn take(self) -> Element<'a, M, R> {
    self.0.0
  }
}

/// Builder for a [`Space`] widget.
#[must_use]
pub struct SpaceBuilder<E> {
  width: Length,
  height: Length,
  elements: E,
}
impl<E> SpaceBuilder<E> {
  fn new(elements: E) -> Self {
    Self {
      width: Length::Shrink,
      height: Length::Shrink,
      elements,
    }
  }
}
impl<'a, E: Elements<'a>> SpaceBuilder<E> {
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

  pub fn done(self) -> E::Output {
    self.elements.push(Space::new(self.width, self.height).into())
  }
}

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<C, E> {
  content: C,
  size: Option<Pixels>,
  elements: E,
}
impl<C, E> TextBuilder<C, E> {
  fn new(content: C, elements: E) -> Self {
    Self { content, size: None, elements }
  }

  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }
}
impl<'a, E: Elements<'a>> TextBuilder<Cow<'a, str>, E> where
  E::Renderer: TextRenderer,
  <E::Renderer as Renderer>::Theme: TextStyleSheet,
{
  pub fn done(self) -> E::Output {
    let mut text = Text::new(self.content);
    if let Some(size) = self.size {
      text = text.size(size);
    }
    self.elements.push(text.into())
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<C, E> {
  contents: C,
  disabled: bool,
  elements: E,
}
impl<C, E> ButtonBuilder<C, E> {
  fn new(contents: C, elements: E) -> Self {
    Self { contents, disabled: false, elements }
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
impl<'a, E: Elements<'a>> ButtonBuilder<Element<'a, (), E::Renderer>, E> where
  <E::Renderer as Renderer>::Theme: ButtonStyleSheet,
{
  pub fn done(self, on_press: impl Fn() -> E::Message + 'a) -> E::Output {
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
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
  elements: E,
}
impl<'a, E: ConsumeElements<'a>> RowBuilder<E> {
  fn new(elements: E) -> Self {
    Self {
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      align_items: Alignment::Start,
      elements
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

  pub fn done(self) -> WidgetBuilder<Elements1<'a, E::Message, E::Renderer>> {
    WidgetBuilder(Elements1(self.take()))
  }
  pub fn take(self) -> Element<'a, E::Message, E::Renderer> {
    Row::with_children(self.elements.consume())
      .spacing(self.spacing)
      .padding(self.padding)
      .width(self.width)
      .height(self.height)
      .align_items(self.align_items)
      .into()
  }
}

mod internal {
  use std::marker::PhantomData;

  use iced::advanced::Renderer;
  use iced::Element;

  use super::WidgetBuilder;

  #[repr(transparent)]
  pub struct Elements0<'a, M, R>(PhantomData<&'a M>, PhantomData<R>);
  impl<'a, M, R> Default for Elements0<'a, M, R> {
    fn default() -> Self { Self(PhantomData::default(), PhantomData::default()) }
  }
  #[repr(transparent)]
  pub struct Elements1<'a, M, R>(pub Element<'a, M, R>);
  #[repr(transparent)]
  pub struct ElementsN<'a, M, R>(pub Vec<Element<'a, M, R>>);

  pub trait Elements<'a> {
    type Message: 'a;
    type Renderer: Renderer + 'a;
    type Output;
    fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Elements<'a> for Elements0<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Output = WidgetBuilder<Elements1<'a, M, R>>;
    fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
      WidgetBuilder(Elements1(element))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Elements<'a> for Elements1<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Output = WidgetBuilder<ElementsN<'a, M, R>>;
    fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
      WidgetBuilder(ElementsN(vec![self.0, element]))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Elements<'a> for ElementsN<'a, M, R> {
    type Message = M;
    type Renderer = R;
    type Output = WidgetBuilder<ElementsN<'a, M, R>>;
    fn push(mut self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
      self.0.push(element);
      WidgetBuilder(self)
    }
  }

  pub trait ConsumeElements<'a> {
    type Message: 'a;
    type Renderer: Renderer + 'a;
    fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
  }
  impl<'a, M: 'a, R: Renderer + 'a> ConsumeElements<'a> for Elements1<'a, M, R> {
    type Message = M;
    type Renderer = R;
    fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
      vec![self.0]
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> ConsumeElements<'a> for ElementsN<'a, M, R> {
    type Message = M;
    type Renderer = R;
    fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
      self.0
    }
  }
}
