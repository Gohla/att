use std::borrow::Cow;
use std::marker::PhantomData;

use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Row, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};

use crate::widget::WidgetExt;

pub fn builder<'a, M: 'a, R: Renderer + 'a>() -> ZeroBuilder<'a, M, R> {
  ZeroBuilder::default()
}

pub trait Builder<'a, M: 'a, R: Renderer + 'a> where
  Self: Sized + PushElement<'a, Message=M, Renderer=R>
{
  fn space(self) -> SpaceBuilder<Self> {
    SpaceBuilder::new(self)
  }
  fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<'a, Self> where
    R: TextRenderer,
    R::Theme: TextStyleSheet,
  {
    TextBuilder::new(content.into(), self)
  }
  fn button(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<Button<'a, (), R>, Self> where
    R::Theme: ButtonStyleSheet,
  {
    ButtonBuilder::new(Button::new(content), self)
  }
  fn element(self, element: impl Into<Element<'a, M, R>>) -> Self::Output
  {
    self.push(element.into())
  }

  fn into_row(self) -> RowBuilder<Self> where
    Self: ConsumeElements<'a, Message=M, Renderer=R>
  {
    RowBuilder::new(self)
  }
}

/// Builder for a [`Space`] widget.
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

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<'a, B> {
  content: Cow<'a, str>,
  size: Option<Pixels>,
  builder: B,
}
impl<'a, B> TextBuilder<'a, B> {
  fn new(content: Cow<'a, str>, builder: B) -> Self {
    Self {
      content,
      size: None,
      builder
    }
  }
}
impl<'a, B: PushElement<'a>> TextBuilder<'a, B> where
  B::Renderer: TextRenderer,
  <B::Renderer as Renderer>::Theme: TextStyleSheet,
{
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }

  pub fn done(self) -> B::Output {
    let mut text = Text::new(self.content);
    if let Some(size) = self.size {
      text = text.size(size);
    }
    self.builder.push(text.into())
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<W, N> {
  widget: W,
  disabled: bool,
  next: N,
}
impl<W, N> ButtonBuilder<W, N> {
  fn new(widget: W, next: N) -> Self { Self { widget, disabled: false, next } }
}
impl<'a, N: PushElement<'a>> ButtonBuilder<Button<'a, (), N::Renderer>, N> where
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

/// Builder for a [`Row`] widget.
#[must_use]
pub struct RowBuilder<N> {
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
  next: N,
}
impl<'a, N: ConsumeElements<'a>> RowBuilder<N> {
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

  pub fn done(self) -> OneBuilder<'a, N::Message, N::Renderer> {
    OneBuilder(self.take())
  }
  pub fn take(self) -> Element<'a, N::Message, N::Renderer> {
    Row::with_children(self.next.consume())
      .spacing(self.spacing)
      .padding(self.padding)
      .width(self.width)
      .height(self.height)
      .align_items(self.align_items)
      .into()
  }
}

// Internals

#[must_use]
pub struct ZeroBuilder<'a, M, R>(PhantomData<&'a M>, PhantomData<R>);
impl<'a, M, R> Default for ZeroBuilder<'a, M, R> {
  fn default() -> Self { Self(PhantomData::default(), PhantomData::default()) }
}
impl<'a, M: 'a, R: Renderer + 'a> Builder<'a, M, R> for ZeroBuilder<'a, M, R> {}

#[must_use]
pub struct OneBuilder<'a, M, R>(Element<'a, M, R>);
impl<'a, M: 'a, R: Renderer + 'a> Builder<'a, M, R> for OneBuilder<'a, M, R> {}
impl<'a, M: 'a, R: Renderer + 'a> OneBuilder<'a, M, R> {
  pub fn take(self) -> Element<'a, M, R> {
    self.0
  }
}

#[must_use]
pub struct ManyBuilder<'a, M, R>(Vec<Element<'a, M, R>>);
impl<'a, M: 'a, R: Renderer + 'a> Builder<'a, M, R> for ManyBuilder<'a, M, R> {}
impl<'a, M: 'a, R: Renderer + 'a> ManyBuilder<'a, M, R> {}

pub trait PushElement<'a> {
  type Message: 'a;
  type Renderer: Renderer + 'a;
  type Output;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output;
}
impl<'a, M: 'a, R: Renderer + 'a> PushElement<'a> for ZeroBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = OneBuilder<'a, M, R>;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    OneBuilder(element)
  }
}
impl<'a, M: 'a, R: Renderer + 'a> PushElement<'a> for OneBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = ManyBuilder<'a, M, R>;
  fn push(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    ManyBuilder(vec![self.0, element])
  }
}
impl<'a, M: 'a, R: Renderer + 'a> PushElement<'a> for ManyBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  type Output = Self;
  fn push(mut self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::Output {
    self.0.push(element);
    self
  }
}

pub trait ConsumeElements<'a> {
  type Message: 'a;
  type Renderer: Renderer + 'a;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>>;
}
impl<'a, M: 'a, R: Renderer + 'a> ConsumeElements<'a> for OneBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    vec![self.0]
  }
}
impl<'a, M: 'a, R: Renderer + 'a> ConsumeElements<'a> for ManyBuilder<'a, M, R> {
  type Message = M;
  type Renderer = R;
  fn consume(self) -> Vec<Element<'a, Self::Message, Self::Renderer>> {
    self.0
  }
}
