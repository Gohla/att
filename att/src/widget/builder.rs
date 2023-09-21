use std::borrow::Cow;
use std::marker::PhantomData;

use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Row, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};

use crate::widget::WidgetExt;

pub fn builder<'a, M>() -> EmptyBuilder<'a, M, iced::Renderer> {
  EmptyBuilder::default()
}

#[must_use]
pub struct EmptyBuilder<'a, M, R>(PhantomData<&'a M>, PhantomData<R>);
impl<'a, M, R> Default for EmptyBuilder<'a, M, R> {
  fn default() -> Self { Self(PhantomData::default(), PhantomData::default()) }
}
impl<'a, M: 'a, R: Renderer> EmptyBuilder<'a, M, R> {
  pub fn space(self) -> SpaceBuilder<Self> {
    SpaceBuilder::new(self)
  }
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<Text<'a, R>, Self> where
    R: TextRenderer,
    R::Theme: TextStyleSheet,
  {
    TextBuilder::new(Text::new(content), self)
  }
  pub fn button(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<Button<'a, (), R>, Self> where
    R::Theme: ButtonStyleSheet,
  {
    ButtonBuilder::new(Button::new(content), self)
  }
  pub fn element(self, element: impl Into<Element<'a, M, R>>) -> OneBuilder<'a, M, R> {
    OneBuilder(element.into())
  }
}

#[must_use]
pub struct OneBuilder<'a, M, R> (Element<'a, M, R>);
impl<'a, M: 'a, R: Renderer> OneBuilder<'a, M, R> {
  pub fn space(self) -> SpaceBuilder<Self> {
    SpaceBuilder::new(self)
  }
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<Text<'a, R>, Self> where
    R: TextRenderer,
    R::Theme: TextStyleSheet,
  {
    TextBuilder::new(Text::new(content), self)
  }
  pub fn button(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<Button<'a, (), R>, Self> where
    R::Theme: ButtonStyleSheet,
  {
    ButtonBuilder::new(Button::new(content), self)
  }
  pub fn element(self, element: impl Into<Element<'a, M, R>>) -> Builder<'a, M, R> {
    Builder { elements: vec![self.0, element.into()] }
  }

  pub fn done(self) -> Element<'a, M, R> {
    self.0
  }
}

#[must_use]
pub struct Builder<'a, M, R> {
  elements: Vec<Element<'a, M, R>>
}
impl<'a, M: 'a, R: Renderer> Builder<'a, M, R> {
  pub fn space(self) -> SpaceBuilder<Self> {
    SpaceBuilder::new(self)
  }
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<Text<'a, R>, Self> where
    R: TextRenderer,
    R::Theme: TextStyleSheet,
  {
    TextBuilder::new(Text::new(content), self)
  }
  pub fn button(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<Button<'a, (), R>, Self> where
    R::Theme: ButtonStyleSheet,
  {
    ButtonBuilder::new(Button::new(content), self)
  }
  pub fn element(mut self, element: impl Into<Element<'a, M, R>>) -> Self {
    self.elements.push(element.into());
    self
  }

  pub fn into_row(self) -> RowBuilder<Self> where
    R: 'a,
  {
    RowBuilder::new(self)
  }
}

#[must_use]
pub struct SpaceBuilder<N> {
  width: Length,
  height: Length,
  next: N,
}
impl<C> SpaceBuilder<C> {
  fn new(next: C) -> Self {
    Self {
      width: Length::Shrink,
      height: Length::Shrink,
      next,
    }
  }
}
impl<'a, N: PushElement<'a>> SpaceBuilder<N> {
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

  pub fn done(self) -> N::Output {
    self.next.push(Space::new(self.width, self.height).into())
  }
}

#[must_use]
pub struct TextBuilder<W, N> {
  widget: W,
  next: N,
}
impl<W, N> TextBuilder<W, N> {
  fn new(widget: W, next: N) -> Self { Self { widget, next } }
}
impl<'a, N: PushElement<'a> + 'a> TextBuilder<Text<'a, N::Renderer>, N> where
  N::Renderer: TextRenderer,
  <N::Renderer as Renderer>::Theme: TextStyleSheet,
{
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.widget = self.widget.size(size);
    self
  }

  pub fn done(self) -> N::Output {
    self.next.push(self.widget.into())
  }
}

#[must_use]
pub struct ButtonBuilder<W, N> {
  widget: W,
  disabled: bool,
  next: N,
}
impl<W, N> ButtonBuilder<W, N> {
  fn new(widget: W, next: N) -> Self { Self { widget, disabled: false, next } }
}
impl<'a, N: PushElement<'a> + 'a> ButtonBuilder<Button<'a, (), N::Renderer>, N> where
  <N::Renderer as Renderer>::Theme: ButtonStyleSheet,
{
  pub fn enabled(mut self) -> Self {
    self.disabled = false;
    self
  }
  pub fn disabled(mut self) -> Self {
    self.disabled = true;
    self
  }

  pub fn done(mut self, on_press: impl Fn() -> N::Message + 'a) -> N::Output {
    if !self.disabled {
      self.widget = self.widget.on_press(());
    }
    let element = self.widget.into_element().map(move |_| on_press());
    self.next.push(element)
  }
}

#[must_use]
pub struct RowBuilder<N> {
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
  next: N,
}
impl<'a, N: ConsumeElements<'a> + 'a> RowBuilder<N> {
  fn new(next: N) -> Self {
    Self {
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      align_items: Alignment::Start,
      next
    }
  }

  /// Sets the horizontal spacing _between_ elements.
  ///
  /// Custom margins per element do not exist in iced. You should use this
  /// method instead! While less flexible, it helps you keep spacing between
  /// elements consistent.
  pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
    self.spacing = amount.into().0;
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

  pub fn done(self) -> OneBuilder<'a, N::Message, N::Renderer> {
    let row = Row::with_children(self.next.consume())
      .spacing(self.spacing)
      .padding(self.padding)
      .width(self.width)
      .height(self.height)
      .align_items(self.align_items)
      ;
    OneBuilder(row.into())
  }
}
// Internals

pub trait PushElement<'a> {
  type Message: 'a;
  type Renderer: Renderer;
  type Output;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output;
}
impl<'a, M: 'a, R: Renderer> PushElement<'a> for EmptyBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = OneBuilder<'a, M, R>;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    OneBuilder(element)
  }
}
impl<'a, M: 'a, R: Renderer> PushElement<'a> for OneBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = Builder<'a, M, R>;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    Self::Output { elements: vec![self.0, element] }
  }
}
impl<'a, M: 'a, R: Renderer> PushElement<'a> for Builder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = Self;
  fn push(mut self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    self.elements.push(element);
    self
  }
}

pub trait ConsumeElements<'a> {
  type Message: 'a;
  type Renderer: Renderer;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
}
impl<'a, M: 'a, R: Renderer> ConsumeElements<'a> for OneBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    vec![self.0]
  }
}
impl<'a, M: 'a, R: Renderer> ConsumeElements<'a> for Builder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    self.elements
  }
}
