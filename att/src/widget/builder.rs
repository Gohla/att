use std::borrow::Cow;

use iced::{Alignment, Color, Element, Length, Padding, Pixels};
use iced::advanced::text::Renderer as TextRenderer;
pub use iced::advanced::widget::text::StyleSheet as TextStyleSheet;
use iced::alignment::{Horizontal, Vertical};
pub use iced::theme::Button as ButtonStyle;
pub use iced::theme::Theme as BuiltinTheme;
use iced::widget::{Button, Column, Container, Row, Rule, Scrollable, Space, Text};
pub use iced::widget::button::StyleSheet as ButtonStyleSheet;
pub use iced::widget::container::{Id as ContainerId, StyleSheet as ContainerStyleSheet};
pub use iced::widget::rule::StyleSheet as RuleStyleSheet;
pub use iced::widget::scrollable::{Id as ScrollableId, StyleSheet as ScrollableStyleSheet};
use iced::widget::scrollable::{Direction, Viewport};
use iced::widget::text::{LineHeight, Shaping};

use internal::{AnyState, Nil, OneState, Heap};

#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<S>(S);
impl<'a, M, R> WidgetBuilder<Nil<Element<'a, M, R>>> {
  pub fn new_stack() -> Self { Self(Nil::default()) }
}
impl<'a, M, R> WidgetBuilder<Heap<Element<'a, M, R>>> {
  pub fn new_heap() -> Self { Self(Heap::new()) }
  pub fn new_heap_with_capacity(capacity: usize) -> Self { Self(Heap::with_capacity(capacity)) }
}
impl<'a, M, R> Default for WidgetBuilder<Nil<Element<'a, M, R>>> {
  fn default() -> Self { Self::new_stack() }
}
impl<'a, S: AnyState<'a>> WidgetBuilder<S> {
  /// Build a [`Space`] widget.
  pub fn space(self) -> SpaceBuilder<S> {
    SpaceBuilder::new(self.0)
  }
  /// Adds a width-filling [`Space`] to this builder.
  pub fn add_space_fill_width(self) -> S::AddBuilder {
    self.space().fill_width().add()
  }
  /// Adds a height-filling [`Space`] to this builder.
  pub fn add_space_fill_height(self) -> S::AddBuilder {
    self.space().fill_height().add()
  }

  /// Build a [`Rule`] widget.
  pub fn rule(self) -> RuleBuilder<S> where
    S::Theme: RuleStyleSheet
  {
    RuleBuilder::new(self.0)
  }
  /// Adds a horizontal [`Rule`] with `height` to this builder.
  pub fn add_horizontal_rule(self, height: impl Into<Pixels>) -> S::AddBuilder where
    S::Theme: RuleStyleSheet
  {
    self.rule().horizontal(height).add()
  }
  /// Adds a vertical [`Rule`] with `width` to this builder.
  pub fn add_vertical_rule(self, width: impl Into<Pixels>) -> S::AddBuilder where
    S::Theme: RuleStyleSheet
  {
    self.rule().vertical(width).add()
  }

  /// Build a [`Text`] widget from `content`.
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<'a, S> where
    S::Renderer: TextRenderer,
    S::Theme: TextStyleSheet
  {
    TextBuilder::new(self.0, content.into())
  }
  /// Build a [`Button`] widget from `content`.
  pub fn button(self, content: impl Into<Element<'a, (), S::Renderer>>) -> ButtonBuilder<'a, S> where
    S::Theme: ButtonStyleSheet
  {
    ButtonBuilder::new(self.0, content.into())
  }

  /// Build an [`Element`] from `element`.
  pub fn element<M: 'a>(self, element: impl Into<Element<'a, M, S::Renderer>>) -> ElementBuilder<'a, S, M> {
    ElementBuilder::new(self.0, element.into())
  }
  /// Adds `element` to this builder.
  pub fn add_element(self, element: impl Into<Element<'a, S::Message, S::Renderer>>) -> S::AddBuilder {
    self.element(element).add()
  }

  /// Build a [`Column`] widget that will consume all elements in this builder.
  pub fn into_col(self) -> ColBuilder<S> {
    ColBuilder::new(self.0)
  }
  /// Build a [`Row`] widget that will consume all elements in this builder.
  pub fn into_row(self) -> RowBuilder<S> {
    RowBuilder::new(self.0)
  }
}
impl<'a, S: OneState<'a>> WidgetBuilder<S> {
  /// Build a [`Scrollable`] widget that will consume the single element in this builder.
  ///
  /// Can only be called when this builder has exactly one widget.
  pub fn into_scrollable(self) -> ScrollableBuilder<'a, S> where
    S::Theme: ScrollableStyleSheet
  {
    ScrollableBuilder::new(self.0)
  }

  /// Build a [`Container`] widget that will consume the single element in this builder.
  ///
  /// Can only be called when this builder has exactly one widget.
  pub fn into_container(self) -> ContainerBuilder<'a, S> where
    S::Theme: ContainerStyleSheet
  {
    ContainerBuilder::new(self.0)
  }

  /// Take the single element out of this builder.
  ///
  /// Can only be called when this builder has exactly one widget.
  pub fn take(self) -> Element<'a, S::Message, S::Renderer> {
    self.0.take()
  }
}
impl<E> WidgetBuilder<Heap<E>> {
  pub fn reserve(mut self, additional: usize) -> Self {
    self.0.reserve(additional);
    self
  }
}

/// Builder for a [`Space`] widget.
#[must_use]
pub struct SpaceBuilder<S> {
  state: S,
  width: Length,
  height: Length,
}
impl<'a, S: AnyState<'a>> SpaceBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
      width: Length::Shrink,
      height: Length::Shrink,
    }
  }

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

  pub fn add(self) -> S::AddBuilder {
    let space = Space::new(self.width, self.height);
    self.state.add(space.into())
  }
}

/// Builder for a [`Rule`] widget.
#[must_use]
pub struct RuleBuilder<S> {
  state: S,
  width_or_height: Pixels,
  is_vertical: bool,
}
impl<'a, S: AnyState<'a>> RuleBuilder<S> where
  S::Theme: RuleStyleSheet
{
  fn new(state: S) -> Self {
    Self {
      state,
      width_or_height: 1.0.into(),
      is_vertical: false,
    }
  }

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

  pub fn add(self) -> S::AddBuilder {
    let rule = if self.is_vertical {
      Rule::vertical(self.width_or_height)
    } else {
      Rule::horizontal(self.width_or_height)
    };
    self.state.add(rule.into())
  }
}

/// Builder for a [`Text`] widget.
#[must_use]
pub struct TextBuilder<'a, S: AnyState<'a>> where
  S::Renderer: TextRenderer,
  S::Theme: TextStyleSheet
{
  state: S,
  text: Text<'a, S::Renderer>
}
impl<'a, S: AnyState<'a>> TextBuilder<'a, S> where
  S::Renderer: TextRenderer,
  S::Theme: TextStyleSheet
{
  fn new(state: S, content: Cow<'a, str>) -> Self {
    Self {
      state,
      text: Text::new(content),
    }
  }

  /// Sets the size of the [`Text`].
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.text = self.text.size(size);
    self
  }
  /// Sets the [`LineHeight`] of the [`Text`].
  pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
    self.text = self.text.line_height(line_height);
    self
  }
  /// Sets the [`Font`] of the [`Text`].
  ///
  /// [`Font`]: S::Renderer::Font
  pub fn font(mut self, font: impl Into<<S::Renderer as TextRenderer>::Font>) -> Self {
    self.text = self.text.font(font);
    self
  }
  /// Sets the [`Style`] of the [`Text`].
  ///
  /// [`Style`]: S::Theme::Style
  pub fn style(mut self, style: impl Into<<S::Theme as TextStyleSheet>::Style>) -> Self {
    self.text = self.text.style(style);
    self
  }
  /// Sets a [`Color`] as the style of the [`Text`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn style_color(self, color: impl Into<Color>) -> Self where
    S: AnyState<'a, Theme=BuiltinTheme>
  {
    self.style(color.into())
  }
  /// Sets the width of the [`Text`] boundaries.
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.text = self.text.width(width);
    self
  }
  /// Sets the height of the [`Text`] boundaries.
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.text = self.text.height(height);
    self
  }
  /// Sets the [`Horizontal`] alignment of the [`Text`].
  pub fn horizontal_alignment(mut self, alignment: Horizontal) -> Self {
    self.text = self.text.horizontal_alignment(alignment);
    self
  }
  /// Sets the [`Vertical`] alignment of the [`Text`].
  pub fn vertical_alignment(mut self, alignment: Vertical) -> Self {
    self.text = self.text.vertical_alignment(alignment);
    self
  }
  /// Sets the [`Shaping`] strategy of the [`Text`].
  pub fn shaping(mut self, shaping: Shaping) -> Self {
    self.text = self.text.shaping(shaping);
    self
  }

  pub fn add(self) -> S::AddBuilder {
    self.state.add(self.text.into())
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<'a, S: AnyState<'a>> where
  S::Theme: ButtonStyleSheet
{
  state: S,
  button: Button<'a, (), S::Renderer>,
  disabled: bool,
}
impl<'a, S: AnyState<'a>> ButtonBuilder<'a, S> where
  S::Theme: ButtonStyleSheet
{
  fn new(state: S, content: Element<'a, (), S::Renderer>) -> Self {
    Self {
      state,
      button: Button::new(content),
      disabled: false,
    }
  }

  /// Sets the width of the [`Button`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.button = self.button.width(width);
    self
  }
  /// Sets the height of the [`Button`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.button = self.button.height(height);
    self
  }
  /// Sets the [`Padding`] of the [`Button`].
  pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    self.button = self.button.padding(padding);
    self
  }
  /// Sets whether the [`Button`] is disabled.
  pub fn disabled(mut self, disabled: bool) -> Self {
    self.disabled = disabled;
    self
  }
  /// Sets the [`Style`] of the [`Button`].
  ///
  /// [`Style`]: S::Theme::Style
  pub fn style(mut self, style: impl Into<<S::Theme as ButtonStyleSheet>::Style>) -> Self {
    self.button = self.button.style(style);
    self
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Primary`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn primary_style(self) -> Self where S: AnyState<'a, Theme=BuiltinTheme> {
    self.style(ButtonStyle::Secondary)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Secondary`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn secondary_style(self) -> Self where S: AnyState<'a, Theme=BuiltinTheme> {
    self.style(ButtonStyle::Secondary)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Positive`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn positive_style(self) -> Self where S: AnyState<'a, Theme=BuiltinTheme> {
    self.style(ButtonStyle::Positive)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Destructive`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn destructive_style(self) -> Self where S: AnyState<'a, Theme=BuiltinTheme> {
    self.style(ButtonStyle::Destructive)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Text`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn text_style(self) -> Self where S: AnyState<'a, Theme=BuiltinTheme> {
    self.style(ButtonStyle::Text)
  }
  /// Sets the style of the [`Button`] to a custom [`ButtonStyleSheet`] implementation.
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn custom_style(self, style_sheet: impl ButtonStyleSheet<Style=BuiltinTheme> + 'static) -> Self where
    S: AnyState<'a, Theme=BuiltinTheme>
  {
    self.style(ButtonStyle::custom(style_sheet))
  }

  /// Sets the function that will be called when the [`Button`] is pressed to `on_press`, then adds the [`Button`] to
  /// the builder and returns the builder.
  ///
  /// Implementation note: the reason for this convoluted way to set the `on_press` function is to avoid a [`Clone`]
  /// requirement for the application message type.
  pub fn add(self, on_press: impl Fn() -> S::Message + 'a) -> S::AddBuilder {
    let mut button = self.button;
    if !self.disabled {
      button = button.on_press(());
    }
    let element = Element::new(button).map(move |_| on_press());
    self.state.add(element)
  }
}

/// Builder for an [`Element`]
#[must_use]
pub struct ElementBuilder<'a, S: AnyState<'a>, M> {
  state: S,
  element: Element<'a, M, S::Renderer>,
}
impl<'a, S: AnyState<'a>, M: 'a> ElementBuilder<'a, S, M> {
  fn new(state: S, element: Element<'a, M, S::Renderer>) -> Self {
    Self { state, element }
  }

  pub fn map(self, f: impl Fn(M) -> S::Message + 'a) -> ElementBuilder<'a, S, S::Message> {
    let element = self.element.map(f);
    ElementBuilder { state: self.state, element }
  }
}
impl<'a, S: AnyState<'a>> ElementBuilder<'a, S, S::Message> {
  pub fn add(self) -> S::AddBuilder {
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
impl<'a, S: AnyState<'a>> ColBuilder<S> {
  pub fn add(self) -> S::ConsumeBuilder {
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
impl<'a, S: AnyState<'a>> RowBuilder<S> {
  pub fn add(self) -> S::ConsumeBuilder {
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

/// Builder for a [`Scrollable`] widget.
#[must_use]
pub struct ScrollableBuilder<'a, S: OneState<'a>> where
  S::Theme: ScrollableStyleSheet
{
  state: S,
  id: Option<ScrollableId>,
  width: Length,
  height: Length,
  direction: Direction,
  on_scroll: Option<Box<dyn Fn(Viewport) -> S::Message + 'a>>,
  style: <S::Theme as ScrollableStyleSheet>::Style,
}
impl<'a, S: OneState<'a>> ScrollableBuilder<'a, S> where
  S::Theme: ScrollableStyleSheet
{
  fn new(state: S) -> Self {
    Self {
      state,
      id: None,
      width: Length::Shrink,
      height: Length::Shrink,
      direction: Default::default(),
      on_scroll: None,
      style: Default::default(),
    }
  }

  /// Sets the [`Id`] of the [`Scrollable`].
  pub fn id(mut self, id: ScrollableId) -> Self {
    self.id = Some(id);
    self
  }
  /// Sets the width of the [`Scrollable`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the height of the [`Scrollable`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the [`Direction`] of the [`Scrollable`] .
  pub fn direction(mut self, direction: Direction) -> Self {
    self.direction = direction;
    self
  }
  /// Sets a function to call when the [`Scrollable`] is scrolled.
  ///
  /// The function takes the [`Viewport`] of the [`Scrollable`].
  pub fn on_scroll(mut self, f: impl Fn(Viewport) -> S::Message + 'a) -> Self {
    self.on_scroll = Some(Box::new(f));
    self
  }
  /// Sets the style of the [`Scrollable`] .
  pub fn style(mut self, style: impl Into<<S::Theme as ScrollableStyleSheet>::Style>) -> Self {
    self.style = style.into();
    self
  }

  pub fn add(self) -> S::MapBuilder {
    self.state.map(|content| {
      let mut scrollable = Scrollable::new(content)
        .width(self.width)
        .height(self.height)
        .direction(self.direction)
        .style(self.style);
      if let Some(id) = self.id {
        scrollable = scrollable.id(id);
      }
      if let Some(on_scroll) = self.on_scroll {
        scrollable = scrollable.on_scroll(on_scroll);
      }
      scrollable.into()
    })
  }
}

/// Builder for a [`Container`] widget.
#[must_use]
pub struct ContainerBuilder<'a, S: OneState<'a>> where
  S::Theme: ContainerStyleSheet
{
  state: S,
  id: Option<ContainerId>,
  padding: Padding,
  width: Length,
  height: Length,
  max_width: f32,
  max_height: f32,
  horizontal_alignment: Horizontal,
  vertical_alignment: Vertical,
  style: <S::Theme as ContainerStyleSheet>::Style,
}
impl<'a, S: OneState<'a>> ContainerBuilder<'a, S> where
  S::Theme: ContainerStyleSheet
{
  fn new(state: S) -> Self {
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
      style: Default::default(),
    }
  }


  /// Sets the [`Id`] of the [`Container`].
  pub fn id(mut self, id: ContainerId) -> Self {
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
  /// Sets the style of the [`Container`].
  pub fn style(mut self, style: impl Into<<S::Theme as ContainerStyleSheet>::Style>) -> Self {
    self.style = style.into();
    self
  }

  pub fn add(self) -> S::MapBuilder {
    self.state.map(|content| {
      let mut container = Container::new(content)
        .padding(self.padding)
        .width(self.width)
        .height(self.height)
        .max_width(self.max_width)
        .max_height(self.max_width)
        .align_x(self.horizontal_alignment)
        .align_y(self.vertical_alignment)
        .style(self.style)
        ;
      if let Some(id) = self.id {
        container = container.id(id);
      }
      container.into()
    })
  }
}


/// Internal state management for widget builder.
mod internal {
  use std::marker::PhantomData;

  use iced::advanced::Renderer;
  use iced::Element;

  use super::WidgetBuilder;

  /// Internal trait for access to element types.
  pub trait Types<'a> {
    /// [`Element`] message type.
    type Message: 'a;
    /// [`Element`] renderer type.
    type Renderer: Renderer<Theme=Self::Theme> + 'a;
    /// Theme type of the [`Self::Renderer`].
    type Theme;
  }

  /// Internal trait for widget builder state of any length, providing add and consume operations.
  pub trait AnyState<'a>: Types<'a> {
    /// Builder type to return from [`Self::add`].
    type AddBuilder;
    /// Add `element` onto `self`, then return a [new builder](Self::AddBuilder) with those elements.
    fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddBuilder;

    /// Builder type to return from [`Self::consume`].
    type ConsumeBuilder;
    /// Consume the [elements](Element) from `self` into a [`Vec`], call `produce` on that [`Vec`] to create a new
    /// [`Element`], then return a [new builder](Self::ConsumeBuilder) with that element.
    fn consume<F>(self, produce: F) -> Self::ConsumeBuilder where
      F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>;
  }

  /// Internal trait for widget builder state of length 1, providing map and take operations.
  pub trait OneState<'a>: Types<'a> {
    /// Builder type to return from [`Self::map`].
    type MapBuilder;
    /// Take the single [`Element`] from `self`, call `map` on that [`Element`] to create a new [`Element`], then return
    /// a [new builder](Self::MapBuilder) with that element.
    fn map<F>(self, map: F) -> Self::MapBuilder where
      F: FnOnce(Element<'a, Self::Message, Self::Renderer>) -> Element<'a, Self::Message, Self::Renderer>;

    /// Take the single [`Element`] from `self` and return it.
    fn take(self) -> Element<'a, Self::Message, Self::Renderer>;
  }


  // Stack implementation: full compile-time safety and zero-cost, but every operation changes the type of the state.

  /// Algebraic stack list constructor.
  pub struct Cons<E, Rest>(E, Rest);
  /// Empty list.
  #[repr(transparent)]
  pub struct Nil<E>(PhantomData<E>);
  impl<E> Default for Nil<E> {
    #[inline]
    fn default() -> Self { Self(PhantomData::default()) }
  }

  /// Internal trait for algebraic stack list operations.
  trait StackList: Sized {
    /// Type of elements in the list.
    type E;
    /// The length of this list.
    const LEN: usize;
    /// Return a new list with `element` added to it.
    #[inline]
    fn add(self, element: Self::E) -> Cons<Self::E, Self> {
      Cons(element, self)
    }
    /// Consume the elements from this list into a [`Vec`].
    #[inline]
    fn consume(self) -> Vec<Self::E> {
      let mut vec = Vec::with_capacity(Self::LEN);
      self.add_to_vec(&mut vec);
      vec
    }
    /// Add the elements of this list into `vec`.
    fn add_to_vec(self, vec: &mut Vec<Self::E>);
  }
  impl<E, Rest: StackList<E=E>> StackList for Cons<E, Rest> {
    type E = E;
    const LEN: usize = 1 + Rest::LEN;
    #[inline]
    fn add_to_vec(self, vec: &mut Vec<Self::E>) {
      // Note: visiting in reverse order to get Vec that is correctly ordered w.r.t. `add`.
      self.1.add_to_vec(vec);
      vec.push(self.0);
    }
  }
  impl<E> StackList for Nil<E> {
    type E = E;
    const LEN: usize = 0;
    #[inline]
    fn add_to_vec(self, _vec: &mut Vec<E>) {}
  }

  impl<'a, M, R, L> Types<'a> for L where
    M: 'a,
    R: Renderer + 'a,
    L: StackList<E=Element<'a, M, R>>
  {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;
  }

  impl<'a, M, R, L> AnyState<'a> for L where
    M: 'a,
    R: Renderer + 'a,
    L: StackList<E=Element<'a, M, R>>
  {
    type AddBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Self>>;
    #[inline]
    fn add(self, element: Element<'a, M, R>) -> Self::AddBuilder {
      WidgetBuilder(StackList::add(self, element))
    }

    type ConsumeBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
    #[inline]
    fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeBuilder {
      let vec = self.consume();
      let element = produce(vec);
      WidgetBuilder(Cons(element, Nil::default()))
    }
  }

  impl<'a, M, R> OneState<'a> for Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>> where
    M: 'a,
    R: Renderer + 'a
  {
    type MapBuilder = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
    #[inline]
    fn map<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapBuilder {
      let element = self.take();
      let element = map(element);
      WidgetBuilder(Cons(element, Nil::default()))
    }

    #[inline]
    fn take(self) -> Element<'a, M, R> {
      self.0
    }
  }


  // Heap implementation: run-time type safety, not zero-cost, but type does not change.

  pub enum Heap<E> {
    Any(Vec<E>),
    One(E, usize),
  }
  impl<E> Heap<E> {
    #[inline]
    pub fn new() -> Self { Self::Any(Vec::new()) }
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self::Any(Vec::with_capacity(capacity)) }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
      match self {
        Heap::Any(ref mut vec) => vec.reserve(additional),
        Heap::One(_, reserve_additional) => *reserve_additional += additional,
      }
    }

    #[inline]
    fn push(self, new_element: E) -> Self {
      match self {
        Heap::Any(mut vec) => {
          vec.push(new_element);
          Heap::Any(vec)
        },
        Heap::One(element, reserve_additional) => {
          let vec = if reserve_additional > 0 {
            let mut vec = Vec::with_capacity(2 + reserve_additional);
            vec.push(element);
            vec.push(new_element);
            vec
          } else {
            vec![element, new_element]
          };
          Heap::Any(vec)
        },
      }
    }
    #[inline]
    fn consume(self) -> Vec<E> {
      match self {
        Heap::Any(vec) => vec,
        Heap::One(element, _) => vec![element], // Note: ignore reserve_additional, since the vec will be consumed as-is.
      }
    }
    #[inline]
    fn take(self) -> (E, usize) {
      match self {
        Heap::Any(mut vec) => {
          let len = vec.len();
          let 1 = len else {
            panic!("builder should have precisely 1 element, but it has {}", len);
          };
          let element = vec.drain(..).next().unwrap();
          (element, 0)
        }
        Heap::One(element, reserve_additional) => (element, reserve_additional),
      }
    }
  }

  impl<'a, M, R> Types<'a> for Heap<Element<'a, M, R>> where
    M: 'a,
    R: Renderer + 'a,
  {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;
  }

  impl<'a, M, R> AnyState<'a> for Heap<Element<'a, M, R>> where
    M: 'a,
    R: Renderer + 'a,
  {
    type AddBuilder = WidgetBuilder<Self>;
    #[inline]
    fn add(self, element: Element<'a, M, R>) -> Self::AddBuilder {
      let heap = self.push(element);
      WidgetBuilder(heap)
    }

    type ConsumeBuilder = WidgetBuilder<Self>;
    #[inline]
    fn consume<F: FnOnce(Vec<Element<'a, M, R>>) -> Element<'a, M, R>>(self, produce: F) -> Self::ConsumeBuilder {
      let vec = self.consume();
      let element = produce(vec);
      WidgetBuilder(Heap::One(element, 0))
    }
  }

  impl<'a, M, R> OneState<'a> for Heap<Element<'a, M, R>> where
    M: 'a,
    R: Renderer + 'a
  {
    type MapBuilder = WidgetBuilder<Self>;
    #[inline]
    fn map<F: FnOnce(Element<'a, M, R>) -> Element<'a, M, R>>(self, map: F) -> Self::MapBuilder {
      let (element, reserve_additional) = self.take();
      let element = map(element);
      WidgetBuilder(Heap::One(element, reserve_additional))
    }

    #[inline]
    fn take(self) -> Element<'a, M, R> {
      self.take().0
    }
  }
}
