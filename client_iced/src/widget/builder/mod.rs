#![allow(unused_imports)]

use std::borrow::Cow;
use std::marker::PhantomData;

use iced::{Alignment, Color, Element, Length, Padding, Pixels};
use iced::advanced::Renderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::alignment::{Horizontal, Vertical};
pub use iced::theme::Button as ButtonStyle;
pub use iced::theme::Theme as BuiltinTheme;
use iced::widget::{Column, Container, Row, Rule, Scrollable, Space, Text, TextInput};
pub use iced::widget::button::StyleSheet as ButtonStyleSheet;
pub use iced::widget::container::{Id as ContainerId, StyleSheet as ContainerStyleSheet};
pub use iced::widget::rule::StyleSheet as RuleStyleSheet;
pub use iced::widget::scrollable::{Id as ScrollableId, StyleSheet as ScrollableStyleSheet};
use iced::widget::scrollable::{Direction, Viewport};
use iced::widget::text::{LineHeight, Shaping};
pub use iced::widget::text::StyleSheet as TextStyleSheet;
pub use iced::widget::text_input::{Icon as TextInputIcon, Id as TextInputId, StyleSheet as TextInputStyleSheet};

use state::{StateAdd, StateConsume, StateMap, StateTake, StateTakeAll};
use state::heap::HeapList;
use state::stack::Nil;
use widget::button::{ButtonActions, ButtonPassthrough, CreateButton};
use widget::text_input::{CreateTextInput, TextInputActions, TextInputPassthrough};

use crate::widget::builder::state::{Elem, State};

mod state;
mod widget;

/// Widget builder.
#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<S>(S);

impl<E: Elem> WidgetBuilder<Nil<E>> {
  /// Create a new stack-allocated widget builder.
  ///
  /// The advantages of a stack-allocated widget builder are:
  /// - It has full compile-time safety: incorrect state is a compilation error.
  /// - It has low run-time overhead: all elements are stored on the stack and are only converted into a [`Vec`] of
  ///   exactly the right size when needed, for example when creating a [`Column`] or [`Row`]. This is equivalent to
  ///   hand-optimized code using `column!` and `row!`, but without needing macros which can break IDE editor services.
  ///   TODO: check to see if it is zero-cost?
  ///
  /// The disadvantage is that every operation changes the type of the builder, and this makes it impossible to use in
  /// some cases. For example, using it in a while loop to continually add elements is not possible. In that case, a
  /// [heap-based][heap] builder can be used.
  ///
  /// [heap]: WidgetBuilder<HeapList<Element<'a, M, T, R>>>::heap()
  pub fn stack() -> Self { Self(Default::default()) }
}

impl<E: Elem> WidgetBuilder<HeapList<E>> {
  /// Create a new heap-allocated widget builder.
  ///
  /// The advantage of a heap-allocated widget builder is that its type never changes. Therefore, it can be used in the
  /// cases where a [stack-allocated][stack] builder cannot be used.
  ///
  /// The disadvantages of a heap-allocated widget builder are:
  /// - It does not have full compile-time safety; some incorrect state is handled at run-time:
  ///   - Attempting to build a [`Scrollable`] or a [`Container`] when there are no elements in the builder panics.
  ///   - Attempting to take the single element out of the builder when there is not exactly 1 element panics.
  /// - It has some run-time overhead: elements are stored on the heap, and some run-time checks are needed. Overhead
  ///   can be minimized by creating the builder with [enough capacity](Self::heap_with_capacity), and by
  ///   [reserving](Self::reserve) additional capacity if needed.
  ///
  /// Prefer a [stack-allocated][stack] builder if possible.
  ///
  /// [stack]: WidgetBuilder<Nil<Element<'a, M, T, R>>>::stack()
  pub fn heap() -> Self { Self(Default::default()) }
  /// Create a new heap-allocated widget builder and reserve `capacity` for elements.
  pub fn heap_with_capacity(capacity: usize) -> Self { Self(HeapList::with_capacity(capacity)) }
}

impl<E: Elem> WidgetBuilder<PhantomData<E>> {
  /// Create a new widget builder that can only be used once to build a single widget.
  pub fn once() -> Self { Self(Default::default()) }
}

impl<S: StateAdd> WidgetBuilder<S> {
  /// Build a [`Space`] widget.
  pub fn space(self) -> SpaceBuilder<S> {
    SpaceBuilder::new(self.0)
  }
  /// Adds a width-filling [`Space`] to this builder.
  pub fn add_space_fill_width(self) -> S::AddOutput where
    S::Element: From<Space>,
  {
    self.space().fill_width().add()
  }
  /// Adds a height-filling [`Space`] to this builder.
  pub fn add_space_fill_height(self) -> S::AddOutput where
    S::Element: From<Space>,
  {
    self.space().fill_height().add()
  }

  /// Build a [`Rule`] widget.
  pub fn rule(self) -> RuleBuilder<S> where
    S::Theme: RuleStyleSheet
  {
    RuleBuilder::new(self.0)
  }
  /// Adds a horizontal [`Rule`] with `height` to this builder.
  pub fn add_horizontal_rule(self, height: impl Into<Pixels>) -> S::AddOutput where
    S::Theme: RuleStyleSheet,
    S::Element: From<Rule>,
  {
    self.rule().horizontal(height).add()
  }
  /// Adds a vertical [`Rule`] with `width` to this builder.
  pub fn add_vertical_rule(self, width: impl Into<Pixels>) -> S::AddOutput where
    S::Theme: RuleStyleSheet,
    S::Element: From<Rule>,
  {
    self.rule().vertical(width).add()
  }

  /// Build a [`Text`] widget from `content`.
  pub fn text<'a>(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<S, Text<'a, S::Theme, S::Renderer>> where
    S::Renderer: TextRenderer,
    S::Theme: TextStyleSheet
  {
    TextBuilder::new(self.0, content.into())
  }
  /// Adds a [`Text`] widget with `content` to this builder.
  pub fn add_text<'a>(self, content: impl Into<Cow<'a, str>>) -> S::AddOutput where
    S::Renderer: TextRenderer,
    S::Theme: TextStyleSheet,
    S::Element: From<Text<'a, S::Theme, S::Renderer>>,
  {
    self.text(content).add()
  }
  /// Build a [`TextInput`] widget from `content`.
  pub fn text_input<'a>(self, placeholder: &'a str, value: &'a str) -> TextInputBuilder<'a, S, TextInputPassthrough> where
    S::Renderer: TextRenderer,
    S::Theme: TextInputStyleSheet
  {
    TextInputBuilder::new(self.0, placeholder, value)
  }
  /// Build a [`Button`] widget from `content`.
  pub fn button<C>(self, content: C) -> ButtonBuilder<S, C, ButtonPassthrough> where
    S::Theme: ButtonStyleSheet
  {
    ButtonBuilder::new(self.0, content)
  }

  /// Build an [`Element`] from `element`.
  pub fn element<'a, M>(self, element: impl Into<Element<'a, M, S::Theme, S::Renderer>>) -> ElementBuilder<'a, S, M> {
    ElementBuilder::new(self.0, element.into())
  }
  /// Adds `element` to this builder.
  pub fn add_element<'a>(self, element: impl Into<Element<'a, S::Message, S::Theme, S::Renderer>>) -> S::AddOutput where
    S::Element: From<Element<'a, S::Message, S::Theme, S::Renderer>>,
  {
    self.element(element).add()
  }
}

impl<'a, S: StateConsume> WidgetBuilder<S> {
  /// Build a [`Column`] widget that will consume all elements in this builder.
  pub fn column(self) -> ColumnBuilder<S> {
    ColumnBuilder::new(self.0)
  }
  /// Build a [`Row`] widget that will consume all elements in this builder.
  pub fn row(self) -> RowBuilder<S> {
    RowBuilder::new(self.0)
  }
}

impl<'a, S: StateMap> WidgetBuilder<S> {
  /// Build a [`Scrollable`] widget that will consume the last element in this builder.
  ///
  /// Can only be called when this builder has at least one element.
  pub fn scrollable(self) -> ScrollableBuilder<'a, S> where
    S::Theme: ScrollableStyleSheet
  {
    ScrollableBuilder::new(self.0)
  }

  /// Build a [`Container`] widget that will consume the last element in this builder.
  ///
  /// Can only be called when this builder has at least one element.
  pub fn container(self) -> ContainerBuilder<S> where
    S::Theme: ContainerStyleSheet
  {
    ContainerBuilder::new(self.0)
  }
}

impl<S: StateTakeAll> WidgetBuilder<S> {
  /// Take a [`Vec`] with all element out of this builder.
  pub fn take_all(self) -> Vec<S::Element> {
    self.0.take_all()
  }
}

impl<S: StateTake> WidgetBuilder<S> {
  /// Take the single element out of this builder.
  ///
  /// Can only be called when this builder has exactly one element.
  pub fn take(self) -> S::Element {
    self.0.take()
  }
}

impl<E> WidgetBuilder<HeapList<E>> {
  /// Reserve space for `additional` elements.
  ///
  /// Can only be called when this is a heap-allocated builder.
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
impl<S: StateAdd> SpaceBuilder<S> {
  fn new(state: S) -> Self {
    Self {
      state,
      width: Length::Shrink,
      height: Length::Shrink,
    }
  }

  /// Sets the `width` of the [`Space`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the `height` of the [`Space`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the `width` of the [`Space`] to `Length::Fill`.
  pub fn fill_width(self) -> Self {
    self.width(Length::Fill)
  }
  /// Sets the `height` of the [`Space`] to `Length::Fill`.
  pub fn fill_height(self) -> Self {
    self.height(Length::Fill)
  }

  /// Adds the [`Space`] widget to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput where
    S::Element: From<Space>
  {
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
impl<'a, S: StateAdd> RuleBuilder<S> where
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

  pub fn add(self) -> S::AddOutput where
    S::Element: From<Rule>,
  {
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
pub struct TextBuilder<S, T> {
  state: S,
  text: T//Text<'a, E::Theme, E::Renderer>
}
impl<'a, S: StateAdd> TextBuilder<S, Text<'a, S::Theme, S::Renderer>> where
  S::Renderer: TextRenderer,
  S::Theme: TextStyleSheet,
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
    S: StateAdd<Theme=BuiltinTheme>
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

  pub fn add(self) -> S::AddOutput where
    S::Element: From<Text<'a, S::Theme, S::Renderer>>,
  {
    self.state.add(self.text)
  }
}

/// Builder for a [`TextInput`] widget.
#[must_use]
pub struct TextInputBuilder<'a, S: StateAdd, A> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet
{
  state: S,
  id: Option<TextInputId>,
  placeholder: &'a str,
  value: &'a str,
  password: bool,
  font: Option<<S::Renderer as TextRenderer>::Font>,
  width: Length,
  padding: Padding,
  size: Option<Pixels>,
  line_height: LineHeight,
  actions: A,
  icon: Option<TextInputIcon<<S::Renderer as TextRenderer>::Font>>,
  style: <S::Theme as TextInputStyleSheet>::Style,
}
impl<'a, S: StateAdd> TextInputBuilder<'a, S, TextInputPassthrough> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet
{
  fn new(state: S, placeholder: &'a str, value: &'a str) -> Self {
    Self {
      state,
      id: None,
      placeholder,
      value,
      password: false,
      font: None,
      width: Length::Fill,
      padding: Padding::new(5.0),
      size: None,
      line_height: LineHeight::default(),
      actions: TextInputPassthrough,
      icon: None,
      style: Default::default(),
    }
  }
}
impl<'a, S: StateAdd, A: TextInputActions<'a, S::Message>> TextInputBuilder<'a, S, A> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet
{
  /// Sets the [`TextInputId`] of the [`TextInput`].
  pub fn id(mut self, id: TextInputId) -> Self {
    self.id = Some(id);
    self
  }
  /// Converts the [`TextInput`] into a secure password input.
  pub fn password(mut self) -> Self {
    self.password = true;
    self
  }
  /// Sets the [`Font`] of the [`TextInput`].
  ///
  /// [`Font`]: S::Renderer::Font
  pub fn font(mut self, font: <S::Renderer as TextRenderer>::Font) -> Self {
    self.font = Some(font);
    self
  }
  /// Sets the [`TextInputIcon`] of the [`TextInput`].
  pub fn icon(mut self, icon: TextInputIcon<<S::Renderer as TextRenderer>::Font>) -> Self {
    self.icon = Some(icon);
    self
  }
  /// Sets the width of the [`TextInput`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the [`Padding`] of the [`TextInput`].
  pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
    self.padding = padding.into();
    self
  }
  /// Sets the text size of the [`TextInput`].
  pub fn size(mut self, size: impl Into<Pixels>) -> Self {
    self.size = Some(size.into());
    self
  }
  /// Sets the [`LineHeight`] of the [`TextInput`].
  pub fn line_height(
    mut self,
    line_height: impl Into<LineHeight>,
  ) -> Self {
    self.line_height = line_height.into();
    self
  }
  /// Sets the style of the [`TextInput`].
  pub fn style(
    mut self,
    style: impl Into<<S::Theme as TextInputStyleSheet>::Style>,
  ) -> Self {
    self.style = style.into();
    self
  }

  /// Sets the function that will be called when text is typed into the [`TextInput`] to `on_input`.
  ///
  /// If this method is not called, the [`TextInput`] will be disabled.
  pub fn on_input<F: Fn(String) -> S::Message + 'a>(self, on_input: F) -> TextInputBuilder<'a, S, A::Change> {
    self.replace_actions(|actions| actions.on_input(on_input))
  }
  /// Sets the function that will be called when text is pasted into the [`TextInput`] to `on_paste`.
  pub fn on_paste<F: Fn(String) -> S::Message + 'a>(self, on_paste: F) -> TextInputBuilder<'a, S, A::Change> {
    self.replace_actions(|actions| actions.on_paste(on_paste))
  }
  /// Sets the function that will be called when the [`TextInput`] is focussed and the enter key is pressed to
  /// `on_paste`.
  pub fn on_submit<F: Fn() -> S::Message + 'a>(self, on_submit: F) -> TextInputBuilder<'a, S, A::Change> {
    self.replace_actions(|actions| actions.on_submit(on_submit))
  }

  fn replace_actions<AA>(self, change: impl FnOnce(A) -> AA) -> TextInputBuilder<'a, S, AA> {
    TextInputBuilder {
      state: self.state,
      id: self.id,
      placeholder: self.placeholder,
      value: self.value,
      password: self.password,
      font: self.font,
      width: self.width,
      padding: self.padding,
      size: self.size,
      line_height: self.line_height,
      actions: change(self.actions),
      icon: self.icon,
      style: self.style
    }
  }
}
impl<'a, S: StateAdd, A: CreateTextInput<'a, S>> TextInputBuilder<'a, S, A> where
  S::Renderer: TextRenderer,
  S::Theme: TextInputStyleSheet,
  S::Element: From<Element<'a, S::Message, S::Theme, S::Renderer>>,
{
  /// Adds the [`TextInput`](iced::widget::TextInput) to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput {
    let element = self.actions.create(&self.placeholder, &self.value, |mut text_input| {
      if let Some(id) = self.id {
        text_input = text_input.id(id);
      }
      if self.password {
        text_input = text_input.password();
      }
      if let Some(font) = self.font {
        text_input = text_input.font(font);
      }
      if let Some(size) = self.size {
        text_input = text_input.size(size);
      }
      if let Some(icon) = self.icon {
        text_input = text_input.icon(icon);
      }
      text_input
        .width(self.width)
        .padding(self.padding)
        .line_height(self.line_height)
        .style(self.style)
    });
    self.state.add(element)
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<S: State, C, A> where
  S::Theme: ButtonStyleSheet
{
  state: S,
  content: C,
  actions: A,
  disabled: bool,
  width: Length,
  height: Length,
  padding: Padding,
  style: <S::Theme as ButtonStyleSheet>::Style,
}
impl<S: State, C> ButtonBuilder<S, C, ButtonPassthrough> where
  S::Theme: ButtonStyleSheet
{
  fn new(state: S, content: C) -> Self {
    Self {
      state,
      content,
      actions: ButtonPassthrough,
      disabled: false,
      width: Length::Shrink,
      height: Length::Shrink,
      padding: 5.0.into(),
      style: Default::default(),
    }
  }
}
impl<'a, S: State, C, A: ButtonActions<'a, S::Message>> ButtonBuilder<S, C, A> where
  S::Theme: ButtonStyleSheet
{
  /// Sets the width of the [`Button`].
  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }
  /// Sets the height of the [`Button`].
  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }
  /// Sets the [`Padding`] of the [`Button`].
  pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    self.padding = padding.into();
    self
  }
  /// Sets the function that will be called when the [`Button`] is pressed to `on_paste`.
  pub fn on_press<F: Fn() -> S::Message + 'a>(self, on_press: F) -> ButtonBuilder<S, C, A::Change> {
    self.replace_actions(|actions| actions.on_press(on_press))
  }
  /// Sets whether the [`Button`] is `disabled`.
  pub fn disabled(mut self, disabled: bool) -> Self {
    self.disabled = disabled;
    self
  }
  /// Sets the [`Style`] of the [`Button`].
  ///
  /// [`Style`]: S::Theme::Style
  pub fn style(mut self, style: impl Into<<S::Theme as ButtonStyleSheet>::Style>) -> Self {
    self.style = style.into();
    self
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Primary`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn primary_style(self) -> Self where S: StateAdd<Theme=BuiltinTheme> {
    self.style(ButtonStyle::Secondary)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Secondary`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn secondary_style(self) -> Self where S: StateAdd<Theme=BuiltinTheme> {
    self.style(ButtonStyle::Secondary)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Positive`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn positive_style(self) -> Self where S: StateAdd<Theme=BuiltinTheme> {
    self.style(ButtonStyle::Positive)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Destructive`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn destructive_style(self) -> Self where S: StateAdd<Theme=BuiltinTheme> {
    self.style(ButtonStyle::Destructive)
  }
  /// Sets the style of the [`Button`] to [`ButtonStyle::Text`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn text_style(self) -> Self where S: StateAdd<Theme=BuiltinTheme> {
    self.style(ButtonStyle::Text)
  }
  /// Sets the style of the [`Button`] to a custom [`ButtonStyleSheet`] implementation.
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn custom_style(self, style_sheet: impl ButtonStyleSheet<Style=BuiltinTheme> + 'static) -> Self where
    S: StateAdd<Theme=BuiltinTheme>
  {
    self.style(ButtonStyle::custom(style_sheet))
  }

  fn replace_actions<AA>(self, change: impl FnOnce(A) -> AA) -> ButtonBuilder<S, C, AA> {
    ButtonBuilder {
      state: self.state,
      content: self.content,
      actions: change(self.actions),
      disabled: self.disabled,
      width: self.width,
      height: self.height,
      padding: self.padding,
      style: self.style,
    }
  }
}
impl<'a, S: StateAdd, C, A: CreateButton<'a, S>> ButtonBuilder<S, C, A> where
  C: Into<Element<'a, A::Message, S::Theme, S::Renderer>>,
  S::Theme: ButtonStyleSheet,
  S::Element: From<Element<'a, S::Message, S::Theme, S::Renderer>>,
{
  /// Adds the [`Button`] to the builder and returns the builder.
  pub fn add(self) -> S::AddOutput {
    let element = self.actions.create(self.content, |button| {
      let mut button = button
        .width(self.width)
        .height(self.height)
        .padding(self.padding)
        .style(self.style);
      if self.disabled {
        button = button.on_press_maybe(None);
      }
      button
    });
    self.state.add(element)
  }
}

/// Builder for an [`Element`].
#[must_use]
pub struct ElementBuilder<'a, S: StateAdd, M> {
  state: S,
  element: Element<'a, M, S::Theme, S::Renderer>,
}
impl<'a, S: StateAdd, M> ElementBuilder<'a, S, M> {
  fn new(state: S, element: Element<'a, M, S::Theme, S::Renderer>) -> Self {
    Self { state, element }
  }

  pub fn map(self, f: impl Fn(M) -> S::Message + 'a) -> ElementBuilder<'a, S, S::Message> where
    M: 'a,
    S::Message: 'a,
    S::Theme: 'a,
    S::Renderer: 'a,
  {
    let element = self.element.map(f);
    ElementBuilder { state: self.state, element }
  }
}
impl<'a, S: StateAdd> ElementBuilder<'a, S, S::Message> where
  S::Element: From<Element<'a, S::Message, S::Theme, S::Renderer>>,
{
  pub fn add(self) -> S::AddOutput {
    self.state.add(self.element)
  }
}

/// Builder for a [`Column`] widget.
#[must_use]
pub struct ColumnBuilder<S> {
  state: S,
  spacing: f32,
  padding: Padding,
  width: Length,
  height: Length,
  max_width: f32,
  align_items: Alignment,
}
impl<S> ColumnBuilder<S> {
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
impl<'a, S: StateConsume> ColumnBuilder<S> where
  Vec<S::Element>: IntoIterator<Item=Element<'a, S::Message, S::Theme, S::Renderer>>, // For `Column::with_children`
  S::Element: From<Column<'a, S::Message, S::Theme, S::Renderer>>,                    // For `.into()`
{
  pub fn add(self) -> S::ConsumeOutput {
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
impl<S> RowBuilder<S> {
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
impl<'a, S: StateConsume> RowBuilder<S> where
  Vec<S::Element>: IntoIterator<Item=Element<'a, S::Message, S::Theme, S::Renderer>>, // For `Row::with_children`
  S::Element: From<Row<'a, S::Message, S::Theme, S::Renderer>>,                       // For `.into()`
{
  pub fn add(self) -> S::ConsumeOutput {
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
pub struct ScrollableBuilder<'a, S: StateMap> where
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
impl<'a, S: StateMap> ScrollableBuilder<'a, S> where
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

  pub fn add(self) -> S::MapOutput where
    Element<'a, S::Message, S::Theme, S::Renderer>: From<S::Element>,    // For `Scrollable::new`
    S::Element: From<Scrollable<'a, S::Message, S::Theme, S::Renderer>>, // For `scrollable.into()`
    S::Message: 'a, // For `scrollable.on_scroll`
  {
    self.state.map_last(|content| {
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
pub struct ContainerBuilder<S: StateMap> where
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
impl<S: StateMap> ContainerBuilder<S> where
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

  pub fn add<'a>(self) -> S::MapOutput where
    Element<'a, S::Message, S::Theme, S::Renderer>: From<S::Element>,   // For `Container::new`
    S::Element: From<Container<'a, S::Message, S::Theme, S::Renderer>>, // For `container.into()`
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
        .style(self.style)
        ;
      if let Some(id) = self.id {
        container = container.id(id);
      }
      container.into()
    })
  }
}
