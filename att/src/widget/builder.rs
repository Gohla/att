use std::borrow::Cow;

use iced::{Element, Length, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Column, Row, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};

use crate::widget::WidgetExt;

pub fn builder<'a, M>() -> Builder<'a, M, iced::Renderer> {
  Builder::default()
}

pub struct Builder<'a, M, R> {
  elements: Vec<Element<'a, M, R>>
}
impl<'a, M> Default for Builder<'a, M, iced::Renderer> {
  fn default() -> Self { Self { elements: Vec::new() } }
}
impl<'a, M: 'a, R: Renderer> Builder<'a, M, R> {
  pub fn new() -> Self { Self { elements: Vec::new() } }

  pub fn space(self) -> SpaceBuilder<Self> {
    SpaceBuilder::new(self)
  }
  pub fn fill_width(self) -> Self {
    SpaceBuilder::new(self).width(Length::Fill).done()
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


  pub fn into_row(self) -> Row<'a, M, R> {
    Row::with_children(self.elements)
  }
  pub fn into_column(self) -> Column<'a, M, R> {
    Column::with_children(self.elements)
  }
}

#[must_use]
pub struct SpaceBuilder<P> {
  width: Length,
  height: Length,
  parent: P,
}
impl<P> SpaceBuilder<P> {
  fn new(parent: P) -> Self {
    Self {
      width: Length::Shrink,
      height: Length::Shrink,
      parent
    }
  }
}
impl<'a, M: 'a, R: Renderer> SpaceBuilder<Builder<'a, M, R>> {
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

  pub fn done(mut self) -> Builder<'a, M, R> {
    self.parent.elements.push(Space::new(self.width, self.height).into());
    self.parent
  }
}

#[must_use]
pub struct TextBuilder<W, P> {
  widget: W,
  parent: P,
}
impl<W, P> TextBuilder<W, P> {
  fn new(widget: W, parent: P) -> Self {
    Self { widget, parent, }
  }
}
impl<'a, M, R> TextBuilder<Text<'a, R>, Builder<'a, M, R>> where
  R: TextRenderer + 'a,
  R::Theme: TextStyleSheet,
{
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.widget = self.widget.size(size);
    self
  }

  pub fn done(mut self) -> Builder<'a, M, R> {
    self.parent.elements.push(self.widget.into());
    self.parent
  }
}

#[must_use]
pub struct ButtonBuilder<W, P> {
  widget: W,
  parent: P,
  disabled: bool,
}
impl<W, P> ButtonBuilder<W, P> {
  fn new(widget: W, parent: P) -> Self {
    Self { widget, parent, disabled: false, }
  }
}
impl<'a, M, R> ButtonBuilder<Button<'a, (), R>, Builder<'a, M, R>> where
  M: 'a,
  R: Renderer + 'a,
  R::Theme: ButtonStyleSheet,
{
  pub fn enabled(mut self) -> Self {
    self.disabled = false;
    self
  }
  pub fn disabled(mut self) -> Self {
    self.disabled = true;
    self
  }

  pub fn done(mut self, on_press: impl Fn() -> M + 'a) -> Builder<'a, M, R> {
    if !self.disabled {
      self.widget = self.widget.on_press(());
    }
    let element = self.widget.into_element().map(move |_| on_press());
    self.parent.elements.push(element);
    self.parent
  }
}
