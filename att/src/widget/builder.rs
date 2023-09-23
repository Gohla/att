use std::borrow::Cow;

use iced::{Alignment, Color, Element, Length, Padding, Pixels};
use iced::advanced::text::Renderer as TextRenderer;
pub use iced::advanced::widget::text::StyleSheet as TextStyleSheet;
use iced::alignment::{Horizontal, Vertical};
pub use iced::theme::Button as ButtonStyle;
pub use iced::theme::Theme as BuiltinTheme;
use iced::widget::{Button, Column, Row, Rule, Space, Text};
pub use iced::widget::button::StyleSheet as ButtonStyleSheet;
pub use iced::widget::rule::StyleSheet as RuleStyleSheet;
use iced::widget::text::{LineHeight, Shaping};

use internal::{Nil, State, Take};

#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<S>(S);
impl<'a, M, R> Default for WidgetBuilder<Nil<Element<'a, M, R>>> {
  fn default() -> Self { Self(Nil::default()) }
}
impl<'a, S: State<'a>> WidgetBuilder<S> {
  /// Build a [`Space`] widget.
  pub fn space(self) -> SpaceBuilder<S> {
    SpaceBuilder::new(self.0)
  }
  /// Adds a width-filling [`Space`] to this builder.
  pub fn add_space_fill_width(self) -> S::AddOutput {
    self.space().fill_width().add()
  }
  /// Adds a height-filling [`Space`] to this builder.
  pub fn add_space_fill_height(self) -> S::AddOutput {
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
    S::Theme: RuleStyleSheet
  {
    self.rule().horizontal(height).add()
  }
  /// Adds a vertical [`Rule`] with `width` to this builder.
  pub fn add_vertical_rule(self, width: impl Into<Pixels>) -> S::AddOutput where
    S::Theme: RuleStyleSheet
  {
    self.rule().vertical(width).add()
  }

  /// Build a [`Text`] widget from `content`.
  pub fn text(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<'a, S> where
    S::Renderer: TextRenderer,
    S::Theme: TextStyleSheet
  {
    TextBuilder::new(self.0, content)
  }
  /// Build a [`Button`] widget from `content`.
  pub fn button(self, content: impl Into<Element<'a, (), S::Renderer>>) -> ButtonBuilder<'a, S> where
    S::Theme: ButtonStyleSheet
  {
    ButtonBuilder::new(self.0, content)
  }

  /// Build an [`Element`] from `element`.
  pub fn element<M: 'a>(self, element: impl Into<Element<'a, M, S::Renderer>>) -> ElementBuilder<'a, S, M> {
    ElementBuilder::new(self.0, element)
  }
  /// Adds `element` to this builder.
  pub fn add_element(self, element: impl Into<Element<'a, S::Message, S::Renderer>>) -> S::AddOutput {
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
impl<S: Take> WidgetBuilder<S> {
  /// Take the single element out of this builder. Can only be called when this builder has exactly one widget.
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
impl<'a, S: State<'a>> SpaceBuilder<S> {
  pub fn add(self) -> S::AddOutput {
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
impl<'a, S: State<'a>> RuleBuilder<S> where
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

  pub fn add(self) -> S::AddOutput {
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
pub struct TextBuilder<'a, S: State<'a>> where
  S::Renderer: TextRenderer,
  S::Theme: TextStyleSheet
{
  state: S,
  text: Text<'a, S::Renderer>
}
impl<'a, S: State<'a>> TextBuilder<'a, S> where
  S::Renderer: TextRenderer,
  S::Theme: TextStyleSheet
{
  fn new(state: S, content: impl Into<Cow<'a, str>>) -> Self {
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
    S: State<'a, Theme=BuiltinTheme>
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

  pub fn add(self) -> S::AddOutput {
    self.state.add(self.text.into())
  }
}

/// Builder for a [`Button`] widget.
#[must_use]
pub struct ButtonBuilder<'a, S: State<'a>> where
  S::Theme: ButtonStyleSheet
{
  state: S,
  button: Button<'a, (), S::Renderer>,
  disabled: bool,
}
impl<'a, S: State<'a>> ButtonBuilder<'a, S> where
  S::Theme: ButtonStyleSheet
{
  fn new(state: S, content: impl Into<Element<'a, (), S::Renderer>>) -> Self {
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
  /// Sets the [`ButtonStyle`] of the [`Button`].
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn style_builtin(self, style: ButtonStyle) -> Self where
    S: State<'a, Theme=BuiltinTheme>
  {
    self.style(style)
  }
  /// Sets the style of the [`Button`] to a custom [`ButtonStyleSheet`] implementation.
  ///
  /// Only available when the [`BuiltinTheme`] is used.
  pub fn style_custom(self, style_sheet: impl ButtonStyleSheet<Style=BuiltinTheme> + 'static) -> Self where
    S: State<'a, Theme=BuiltinTheme>
  {
    self.style(ButtonStyle::custom(style_sheet))
  }

  /// Sets the function that will be called when the [`Button`] is pressed to `on_press`, then adds the [`Button`] to
  /// the builder and returns the builder.
  ///
  /// Implementation note: the reason for this convoluted way to set the `on_press` function is to avoid a [`Clone`]
  /// requirement for the application message type.
  pub fn add(self, on_press: impl Fn() -> S::Message + 'a) -> S::AddOutput {
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
pub struct ElementBuilder<'a, S: State<'a>, M> {
  state: S,
  element: Element<'a, M, S::Renderer>,
}
impl<'a, S: State<'a>, M: 'a> ElementBuilder<'a, S, M> {
  fn new(state: S, element: impl Into<Element<'a, M, S::Renderer>>) -> Self {
    Self { state, element: element.into() }
  }

  pub fn map(self, f: impl Fn(M) -> S::Message + 'a) -> ElementBuilder<'a, S, S::Message> {
    let element = self.element.map(f);
    ElementBuilder { state: self.state, element }
  }
}
impl<'a, S: State<'a>> ElementBuilder<'a, S, S::Message> {
  pub fn add(self) -> S::AddOutput {
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
impl<'a, S: State<'a>> ColBuilder<S> {
  pub fn consume(self) -> S::ConsumeOutput {
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
impl<'a, S: State<'a>> RowBuilder<S> {
  pub fn consume(self) -> S::ConsumeOutput {
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

  use crate::widget::builder::WidgetBuilder;

  /// Algebraic list constructor.
  pub struct Cons<T, Rest>(T, Rest);
  /// Empty list constructor.
  #[repr(transparent)]
  pub struct Nil<T>(PhantomData<T>);
  impl<T> Default for Nil<T> {
    fn default() -> Self { Self(PhantomData::default()) }
  }

  /// Trait for adding to algebraic list and converting it into a [`Vec`].
  pub trait List<T>: Sized {
    #[inline]
    fn add(self, val: T) -> Cons<T, Self> {
      Cons(val, self)
    }
    #[inline]
    fn into_vec(self) -> Vec<T> {
      let mut vec = Vec::with_capacity(Self::LEN);
      self.add_to_vec(&mut vec);
      vec
    }

    const LEN: usize;
    fn add_to_vec(self, vec: &mut Vec<T>);
  }
  impl<T, Rest: List<T>> List<T> for Cons<T, Rest> {
    const LEN: usize = 1 + Rest::LEN;
    #[inline]
    fn add_to_vec(self, vec: &mut Vec<T>) {
      // Note: visiting in reverse order to get Vec that is correctly ordered w.r.t. `add`.
      self.1.add_to_vec(vec);
      vec.push(self.0);
    }
  }
  impl<T> List<T> for Nil<T> {
    const LEN: usize = 0;
    #[inline]
    fn add_to_vec(self, _vec: &mut Vec<T>) {}
  }

  pub trait State<'a> {
    /// [`Element`] message type.
    type Message: 'a;
    /// [`Element`] renderer type.
    type Renderer: Renderer<Theme=Self::Theme> + 'a;
    /// Theme type of the [`Self::Renderer`].
    type Theme;

    type AddOutput;
    fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddOutput;

    type ConsumeOutput;
    fn consume<F>(self, consume: F) -> Self::ConsumeOutput where
      F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>;
  }
  impl<'a, M: 'a, R: Renderer + 'a, L: List<Element<'a, M, R>>> State<'a> for Cons<Element<'a, M, R>, L> {
    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;

    type AddOutput = WidgetBuilder<Cons<Element<'a, M, R>, Self>>;
    fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddOutput {
      WidgetBuilder(List::add(self, element))
    }

    type ConsumeOutput = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
    fn consume<F>(self, consume: F) -> Self::ConsumeOutput
      where F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>
    {
      let vec = self.into_vec();
      let element = consume(vec);
      WidgetBuilder(Cons(element, Nil::default()))
    }
  }
  impl<'a, M: 'a, R: Renderer + 'a> State<'a> for Nil<Element<'a, M, R>> {
    // TODO: can we avoid implementing State for both Cons and Nil? If we make the impl generic over List, we get an
    //       error that M and R are not used :(

    type Message = M;
    type Renderer = R;
    type Theme = R::Theme;

    type AddOutput = WidgetBuilder<Cons<Element<'a, M, R>, Self>>;
    fn add(self, element: Element<'a, Self::Message, Self::Renderer>) -> Self::AddOutput {
      WidgetBuilder(List::add(self, element))
    }

    type ConsumeOutput = WidgetBuilder<Cons<Element<'a, M, R>, Nil<Element<'a, M, R>>>>;
    fn consume<F>(self, consume: F) -> Self::ConsumeOutput where
      F: FnOnce(Vec<Element<'a, Self::Message, Self::Renderer>>) -> Element<'a, Self::Message, Self::Renderer>
    {
      let vec = self.into_vec();
      let element = consume(vec);
      WidgetBuilder(Cons(element, Nil::default()))
    }
  }

  pub trait Take {
    type Element;
    fn take(self) -> Self::Element;
  }
  impl<T> Take for Cons<T, Nil<T>> {
    type Element = T;
    fn take(self) -> Self::Element {
      self.0
    }
  }
}
