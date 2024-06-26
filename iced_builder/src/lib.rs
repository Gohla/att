use std::borrow::Cow;
use std::marker::PhantomData;

use iced::advanced::text::Renderer as TextRenderer;
use iced::Pixels;
use iced::widget::{button, container, Rule, rule, scrollable, Space, Text, text, text_input};

use internal::state::{Elem, ElemM, StateAppend, StateMap, StateReduce, StateTake, StateTakeAll};
use internal::state::heap::HeapList;
use internal::state::stack::Nil;
use widget::button::ButtonBuilder;
use widget::column::ColumnBuilder;
use widget::container::ContainerBuilder;
use widget::element::ElementBuilder;
use widget::row::RowBuilder;
use widget::rule::RuleBuilder;
use widget::scrollable::ScrollableBuilder;
use widget::space::SpaceBuilder;
use widget::text::TextBuilder;
use widget::text_input::TextInputBuilder;

pub mod widget;
mod internal;

/// Widget builder.
#[repr(transparent)]
#[must_use]
pub struct WidgetBuilder<S>(S);

impl<E> WidgetBuilder<Nil<E>> {
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
  /// [heap]: WidgetBuilder<HeapList<E>>::heap()
  pub fn stack() -> Self {
    Self(Nil::default())
  }
}

impl<E> WidgetBuilder<HeapList<E>> {
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
  /// [stack]: WidgetBuilder<Nil<E>>::stack()
  pub fn heap() -> Self {
    Self(HeapList::default())
  }

  /// Create a new heap-allocated widget builder and reserve `capacity` for elements.
  pub fn heap_with_capacity(capacity: usize) -> Self {
    Self(HeapList::with_capacity(capacity))
  }
}

impl<E> WidgetBuilder<PhantomData<E>> {
  /// Create a new widget builder that can only be used once to build a single widget.
  pub fn once() -> Self {
    Self(PhantomData::default())
  }
}

impl<S: StateAppend> WidgetBuilder<S> {
  /// Build a [`Space`] widget.
  pub fn space(self) -> SpaceBuilder<S> {
    SpaceBuilder::new(self.0)
  }

  /// Adds a width-filling [`Space`] to this builder.
  pub fn add_space_fill_width(self) -> S::AddOutput where
    Space: Into<S::Element>,
  {
    self.space().fill_width().add()
  }

  /// Adds a height-filling [`Space`] to this builder.
  pub fn add_space_fill_height(self) -> S::AddOutput where
    Space: Into<S::Element>,
  {
    self.space().fill_height().add()
  }


  /// Build a [`Rule`] widget.
  pub fn rule(self) -> RuleBuilder<S> where
    S::Theme: rule::Catalog,
  {
    RuleBuilder::new(self.0)
  }

  /// Adds a horizontal [`Rule`] with `height` to this builder.
  pub fn add_horizontal_rule<'a>(self, height: impl Into<Pixels>) -> S::AddOutput where
    S::Theme: rule::Catalog,
    Rule<'a>: Into<S::Element>,
  {
    self.rule().horizontal(height).add()
  }

  /// Adds a vertical [`Rule`] with `width` to this builder.
  pub fn add_vertical_rule<'a>(self, width: impl Into<Pixels>) -> S::AddOutput where
    S::Theme: rule::Catalog,
    Rule<'a>: Into<S::Element>,
  {
    self.rule().vertical(width).add()
  }


  /// Build a [`Text`] widget from `content`.
  pub fn text<'a>(self, content: impl Into<Cow<'a, str>>) -> TextBuilder<'a, S> where
    S::Renderer: TextRenderer,
    S::Theme: text::Catalog
  {
    TextBuilder::new(self.0, content.into())
  }

  /// Adds a [`Text`] widget with `content` to this builder.
  pub fn add_text<'a>(self, content: impl Into<Cow<'a, str>>) -> S::AddOutput where
    S::Renderer: TextRenderer,
    S::Theme: text::Catalog,
    Text<'a, S::Theme, S::Renderer>: Into<S::Element>,
  {
    self.text(content).add()
  }

  /// Build a [`TextInput`](iced::widget::TextInput) widget from `content`.
  pub fn text_input<'a>(self, placeholder: &'a str, value: &'a str) -> TextInputBuilder<'a, S> where
    S::Renderer: TextRenderer,
    S::Theme: text_input::Catalog
  {
    TextInputBuilder::new(self.0, placeholder, value)
  }

  /// Build a [`Button`](iced::widget::Button) widget from `content`.
  pub fn button<'a, C>(self, content: C) -> ButtonBuilder<'a, S, C> where
    S::Theme: button::Catalog
  {
    ButtonBuilder::new(self.0, content)
  }


  /// Build an [`Element`](iced::Element) from `element`.
  pub fn element<'a, M>(self, element: impl Into<ElemM<'a, S, M>>) -> ElementBuilder<'a, S, M> {
    ElementBuilder::new(self.0, element.into())
  }

  /// Adds `element` to this builder.
  pub fn add_element<'a>(self, element: impl Into<Elem<'a, S>>) -> S::AddOutput where
    Elem<'a, S>: Into<S::Element>,
  {
    self.element(element).add()
  }
}

impl<S: StateReduce> WidgetBuilder<S> {
  /// Build a [`Column`](iced::widget::Column) widget that will consume all elements in this builder.
  pub fn column(self) -> ColumnBuilder<S> {
    ColumnBuilder::new(self.0)
  }

  /// Build a [`Row`](iced::widget::Column) widget that will consume all elements in this builder.
  pub fn row(self) -> RowBuilder<S> {
    RowBuilder::new(self.0)
  }
}

impl<S: StateMap> WidgetBuilder<S> {
  /// Build a [`Scrollable`](iced::widget::Scrollable) widget that will consume the last element in this builder.
  ///
  /// Can only be called when this builder has at least one element.
  pub fn scrollable<'a>(self) -> ScrollableBuilder<'a, S> where
    S::Theme: scrollable::Catalog,
  {
    ScrollableBuilder::new(self.0)
  }

  /// Build a [`Container`](iced::widget::Container) widget that will consume the last element in this builder.
  ///
  /// Can only be called when this builder has at least one element.
  pub fn container<'a>(self) -> ContainerBuilder<'a, S> where
    S::Theme: container::Catalog,
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
