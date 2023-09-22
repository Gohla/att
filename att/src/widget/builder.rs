use std::borrow::Cow;

use iced::{Alignment, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::text::{StyleSheet as TextStyleSheet, Text};
use iced::widget::{Column, Row, Rule, Space};
use iced::widget::button::{Button, StyleSheet as ButtonStyleSheet};
use iced::widget::rule::StyleSheet as RuleStyleSheet;

use internal::{Add, Consume, State, Take};

#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<S>(S);
impl<'a, M, R> Default for WidgetBuilder<internal::Empty<'a, M, R>> {
  fn default() -> Self { Self(Default::default()) }
}

// Builder methods for building standalone widgets.
impl<'a, S: State<'a>> WidgetBuilder<S> {
  pub fn space(self) -> SpaceBuilder<S> {
    SpaceBuilder::new(self.0)
  }
  pub fn rule(self) -> RuleBuilder<S> {
    RuleBuilder::new(self.0)
  }
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<S, Cow<'a, str>> {
    TextBuilder::new(self.0, content.into())
  }
  pub fn button<R>(self, content: impl Into<Element<'a, (), R>>) -> ButtonBuilder<S, Element<'a, (), R>, ButtonStyle<'a, S>> {
    ButtonBuilder::new(self.0, content.into())
  }
  pub fn element<M, R>(self, element: impl Into<Element<'a, M, R>>) -> ElementBuilder<S, Element<'a, M, R>> {
    ElementBuilder::new(self.0, element.into())
  }
}
// Builder methods for directly adding common widgets.
impl<'a, S: Add<'a>> WidgetBuilder<S> {
  pub fn add_space_fill_width(self) -> S::Builder {
    self.space().fill_width().add()
  }
  pub fn add_space_fill_height(self) -> S::Builder {
    self.space().fill_height().add()
  }
  pub fn add_horizontal_rule(self, height: impl Into<Pixels>) -> S::Builder where
    <S::Renderer as Renderer>::Theme: RuleStyleSheet,
  {
    self.rule().horizontal(height).add()
  }
  pub fn add_vertical_rule(self, width: impl Into<Pixels>) -> S::Builder where
    <S::Renderer as Renderer>::Theme: RuleStyleSheet,
  {
    self.rule().vertical(width).add()
  }
  pub fn add_element(self, element: impl Into<Element<'a, S::Message, S::Renderer>>) -> S::Builder {
    self.element(element).add()
  }
}
// Builder methods for creating container widgets with children widgets, such as columns and rows.
impl<'a, S: Consume<'a>> WidgetBuilder<S> {
  pub fn into_col(self) -> ColBuilder<S> {
    ColBuilder::new(self.0)
  }
  pub fn into_row(self) -> RowBuilder<S> {
    RowBuilder::new(self.0)
  }
}
// Builder methods for taking the result of building.
impl<'a, S: Take<'a>> WidgetBuilder<S> {
  pub fn take(self) -> S::Element {
    self.0.take()
  }
}

/// Builder for a [`Space`] widget.
#[must_use]
pub struct SpaceBuilder<S> {
  state: S,
  width: Length,
  height: Length,
}
impl<S> SpaceBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
      width: Length::Shrink,
      height: Length::Shrink,
    }
  }
}
impl<S> SpaceBuilder<S> {
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
impl<'a, S: Add<'a>> SpaceBuilder<S> {
  pub fn add(self) -> S::Builder {
    let space = Space::new(self.width, self.height);
    self.state.add(space)
  }
}

/// Builder for a [`Rule`] widget.
#[must_use]
pub struct RuleBuilder<S> {
  state: S,
  width_or_height: Pixels,
  is_vertical: bool,
}
impl<S> RuleBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
      width_or_height: 1.0.into(),
      is_vertical: false,
    }
  }
}
impl<S> RuleBuilder<S> {
  pub fn horizontal(mut self, height: impl Into<Pixels>) -> Self {
    self.width_or_height = height.into();
    self.is_vertical = false;
    self
  }
  pub fn vertical(mut self, width: impl Into<Pixels>) -> Self {
    self.width_or_height = width.into();
    self.is_vertical = true;
    self
  }
}
impl<'a, S: Add<'a>> RuleBuilder<S> where
  <S::Renderer as Renderer>::Theme: RuleStyleSheet,
{
  pub fn add(self) -> S::Builder {
    let rule = if self.is_vertical {
      Rule::vertical(self.width_or_height)
    } else {
      Rule::horizontal(self.width_or_height)
    };
    self.state.add(rule)
  }
}

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<S, C> {
  state: S,
  content: C,
  size: Option<Pixels>,
}
impl<S, C> TextBuilder<S, C> {
  fn new(state: S, content: C) -> Self {
    Self { state, content, size: None }
  }

  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }
}
impl<'a, S: Add<'a>> TextBuilder<S, Cow<'a, str>> where
  S::Renderer: TextRenderer,
  <S::Renderer as Renderer>::Theme: TextStyleSheet,
{
  pub fn add(self) -> S::Builder {
    let mut text = Text::new(self.content);
    if let Some(size) = self.size {
      text = text.size(size);
    }
    self.state.add(text)
  }
}

/// Type of styles for [`Button`].
pub type ButtonStyle<'a, S> = <<S as State<'a>>::Theme as ButtonStyleSheet>::Style;

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<S, C, Y> {
  state: S,
  content: C,
  disabled: bool,
  width: Length,
  height: Length,
  padding: Padding,
  style: Y,
}
impl<'a, S: State<'a>, C> ButtonBuilder<S, C, ButtonStyle<'a, S>> {
  fn new(state: S, content: C) -> Self {
    Self {
      state,
      content,
      disabled: false,
      width: Length::Shrink,
      height: Length::Shrink,
      padding: 5.0.into(),
      style: Default::default(),
    }
  }

  /// Enables this [`Button`].
  pub fn enabled(mut self) -> Self {
    self.disabled = false;
    self
  }
  /// Disables this [`Button`].
  pub fn disabled(mut self) -> Self {
    self.disabled = true;
    self
  }
  /// Sets the width of this [`Button`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the height of this [`Button`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the [`Padding`] of this [`Button`].
  pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    self.padding = padding.into();
    self
  }
  /// Sets the style of this [`Button`].
  pub fn style(mut self, style: ButtonStyle<'a, S>) -> Self {
    self.style = style;
    self
  }
}
impl<'a, S: Add<'a>> ButtonBuilder<S, Element<'a, (), S::Renderer>, ButtonStyle<'a, S>> {
  pub fn add(self, on_press: impl Fn() -> S::Message + 'a) -> S::Builder {
    let mut button = Button::new(self.content)
      .width(self.width)
      .height(self.height)
      .padding(self.padding)
      .style(self.style);
    if !self.disabled {
      button = button.on_press(());
    }
    let element = Element::new(button).map(move |_| on_press());
    self.state.add(element)
  }
}

/// Builder for an [`Element`]
#[must_use]
pub struct ElementBuilder<S, E> {
  state: S,
  element: E,
}
impl<S, E> ElementBuilder<S, E> {
  fn new(state: S, element: E) -> Self {
    Self { state, element }
  }
}
impl<'a, S: Add<'a>, M: 'a> ElementBuilder<S, Element<'a, M, S::Renderer>> {
  pub fn map(self, f: impl Fn(M) -> S::Message + 'a) -> ElementBuilder<S, Element<'a, S::Message, S::Renderer>> {
    let element = self.element.map(f);
    ElementBuilder { state: self.state, element }
  }
}
impl<'a, S: Add<'a>> ElementBuilder<S, Element<'a, S::Message, S::Renderer>> {
  pub fn add(self) -> S::Builder {
    // TODO: calling this causes infinite recursion in Jetbrains Rust plugin!
    self.state.add(self.element)
  }
}

/// Builder for a [`Column`] widget.
#[must_use]
pub struct ColBuilder<S> {
  state: S,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  max_width: f32,
  align_items: Alignment,
}
impl<'a, S> ColBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
      spacing: 0.0,
      padding: Padding::ZERO,
      width: Length::Shrink,
      height: Length::Shrink,
      max_width: f32::INFINITY,
      align_items: Alignment::Start,
    }
  }

  /// Sets the vertical spacing _between_ elements.
  ///
  /// Custom margins per element do not exist in iced. You should use this
  /// method instead! While less flexible, it helps you keep spacing between
  /// elements consistent.
  pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
    self.spacing = amount.into().0;
    self
  }
  /// Sets the [`Padding`] of the [`Column`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }

  /// Sets the width of the [`Column`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the height of the [`Column`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the maximum width of the [`Column`].
  pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
    self.max_width = max_width.into().0;
    self
  }
  /// Sets the width of the [`Column`] to [`Length::Fill`].
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }
  /// Sets the height of the [`Column`] to [`Length::Fill`].
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }
  /// Sets the width and height of the [`Column`] to [`Length::Fill`].
  pub fn fill(self) -> Self {
    self.fill_width().fill_height()
  }

  /// Sets the horizontal alignment of the contents of the [`Column`] .
  pub fn align_items(mut self, align: Alignment) -> Self {
    self.align_items = align;
    self
  }
  /// Sets the horizontal alignment of the contents of the [`Column`] to [`Align::Center`].
  pub fn align_center(self) -> Self {
    self.align_items(Alignment::Center)
  }
}
impl<'a, S: Consume<'a>> ColBuilder<S> {
  pub fn consume(self) -> S::Builder {
    self.state.consume(|vec| {
      Column::with_children(vec)
        .spacing(self.spacing)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .max_width(self.max_width)
        .align_items(self.align_items)
        .into()
    })
  }
}

/// Builder for a [`Row`] widget.
#[must_use]
pub struct RowBuilder<S> {
  state: S,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  align_items: Alignment,
}
impl<'a, S> RowBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
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
  /// Sets the width of the [`Row`] to [`Length::Fill`].
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }
  /// Sets the height of the [`Row`] to [`Length::Fill`].
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }
  /// Sets the width and height of the [`Row`] to [`Length::Fill`].
  pub fn fill(self) -> Self {
    self.fill_width().fill_height()
  }

  /// Sets the vertical alignment of the contents of the [`Row`].
  pub fn align_items(mut self, align: Alignment) -> Self {
    self.align_items = align;
    self
  }
  /// Sets the vertical alignment of the contents of the [`Row`] to [`Alignment::Center`].
  pub fn align_center(self) -> Self {
    self.align_items(Alignment::Center)
  }
}
impl<'a, S: Consume<'a>> RowBuilder<S> {
  pub fn consume(self) -> S::Builder {
    self.state.consume(|vec| {
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
  use iced::widget::button::StyleSheet as ButtonStyleSheet;

  use super::WidgetBuilder;

  /// State with exactly 0 elements.
  #[repr(transparent)]
  pub struct Empty<'a, M, R>(PhantomData<&'a M>, PhantomData<R>);
  impl<'a, M, R> Default for Empty<'a, M, R> {
    fn default() -> Self { Self(PhantomData::default(), PhantomData::default()) }
  }
  /// State with exactly 1 element.
  #[repr(transparent)]
  pub struct One<'a, M, R>(Element<'a, M, R>);
  /// State with more than 1 element.
  #[repr(transparent)]
  pub struct Many<'a, M, R>(Vec<Element<'a, M, R>>);

  pub trait State<'a> {
    /// [`Element`] message type.
    type Message: 'a;
    /// [`Element`] renderer type.
    type Renderer: Renderer<Theme=Self::Theme> + 'a;
    /// Theme type of the [`Self::Renderer`].
    type Theme: ThemeRequirements;
  }
  impl<'a, M: 'a, R: Renderer + 'a> State<'a> for Empty<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;
  }
  impl<'a, M: 'a, R: Renderer + 'a> State<'a> for One<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;
  }
  impl<'a, M: 'a, R: Renderer + 'a> State<'a> for Many<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;
  }

  pub trait ThemeRequirements: ButtonStyleSheet {}
  impl<T: ButtonStyleSheet> ThemeRequirements for T {}

  /// Internal trait for adding elements onto the state of a widget builder.
  pub trait Add<'a>: State<'a> {
    /// Builder produced by [`push`].
    type Builder;
    /// Push the [`Element`] produced by `into_element` onto `self`, then return a new [builder](Self::Builder) with
    /// those elements.
    fn add<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Add<'a> for Empty<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn add<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      WidgetBuilder(One(element))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Add<'a> for One<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Builder = WidgetBuilder<Many<'a, M, R>>;
    fn add<I: Into<Element<'a, Self::Message, Self::Renderer>>>(self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      let elements = vec![self.0, element];
      WidgetBuilder(Many(elements))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Add<'a> for Many<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Builder = WidgetBuilder<Many<'a, M, R>>;
    fn add<I: Into<Element<'a, Self::Message, Self::Renderer>>>(mut self, into_element: I) -> Self::Builder {
      let element = into_element.into();
      self.0.push(element);
      WidgetBuilder(self)
    }
  }

  /// Internal trait for consuming elements from the state of a widget builder.
  pub trait Consume<'a>: State<'a> {
    /// Builder produced by [`consume`].
    type Builder;
    /// Consume the [elements](Element) from `self` into a [`Vec`], call `produce` on that [`Vec`] to create a new
    /// [`Element`], then return a new [builder](Self::Builder) with that element.
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder;
  }
  impl<'a, M: 'a, R: Renderer + 'a> Consume<'a> for One<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder {
      let elements = vec![self.0];
      let new_element = produce(elements);
      WidgetBuilder(One(new_element))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> Consume<'a> for Many<'a, M, R> where
    R::Theme: ThemeRequirements
  {
    type Builder = WidgetBuilder<One<'a, M, R>>;
    fn consume<F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>>(self, produce: F) -> Self::Builder {
      let elements = self.0;
      let new_element = produce(elements);
      WidgetBuilder(One(new_element))
    }
  }

  /// Internal trait for taking a single element from the state of a widget builder.
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
