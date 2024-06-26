use iced::Length;
use iced::widget::{scrollable, Scrollable};
use iced::widget::scrollable::{Direction, Viewport};

use crate::internal::state::{Elem, StateMap};
use crate::internal::util::{TNone, TOption, TOptionFn, TSome};

/// Builder for a [`Scrollable`] widget.
#[must_use]
pub struct ScrollableBuilder<'a, S: StateMap, FS = TNone> where
  S::Theme: scrollable::Catalog
{
  state: S,
  id: Option<scrollable::Id>,
  width: Length,
  height: Length,
  direction: Direction,
  on_scroll: FS,
  class: <S::Theme as scrollable::Catalog>::Class<'a>,
}

impl<'a, S: StateMap> ScrollableBuilder<'a, S> where
  S::Theme: scrollable::Catalog
{
  pub(crate) fn new(state: S) -> Self {
    Self {
      state,
      id: None,
      width: Length::Shrink,
      height: Length::Shrink,
      direction: Default::default(),
      on_scroll: TNone,
      class: <S::Theme as scrollable::Catalog>::default(),
    }
  }
}

impl<'a, S: StateMap, FS> ScrollableBuilder<'a, S, FS> where
  S::Theme: scrollable::Catalog
{
  /// Sets the [`scrollable::Id`] of the [`Scrollable`].
  pub fn id(mut self, id: scrollable::Id) -> Self {
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
  pub fn on_scroll<F: Fn(Viewport) -> S::Message + 'a>(self, on_scroll: F) -> ScrollableBuilder<'a, S, TSome<F>> {
    ScrollableBuilder {
      state: self.state,
      id: self.id,
      width: self.width,
      height: self.height,
      direction: self.direction,
      on_scroll: TSome(on_scroll),
      class: self.class,
    }
  }


  /// Sets the `styler` function of the [`Scrollable`] .
  pub fn style(mut self, styler: impl Fn(&S::Theme, scrollable::Status) -> scrollable::Style + 'a) -> Self where
    <S::Theme as scrollable::Catalog>::Class<'a>: From<scrollable::StyleFn<'a, S::Theme>>,
  {
    self.class = (Box::new(styler) as scrollable::StyleFn<'a, S::Theme>).into();
    self
  }

  /// Sets the `class` of the [`Scrollable`] .
  pub fn class(mut self, class: impl Into<<S::Theme as scrollable::Catalog>::Class<'a>>) -> Self {
    self.class = class.into();
    self
  }


  /// Takes the last element out of the builder, creates the [`Scrollable`] with that element, then adds the scrollable
  /// to the builder and returns the builder.
  pub fn add(self) -> S::MapOutput where
    S::Element: Into<Elem<'a, S>>, // For `Scrollable::new`
    Scrollable<'a, S::Message, S::Theme, S::Renderer>: Into<S::Element>, // For `scrollable.into()`
    S::Message: 'a, // For `scrollable.on_scroll`
    FS: TOptionFn<'a, Viewport, S::Message> + 'a
  {
    self.state.map_last(|content| {
      let mut scrollable = Scrollable::with_direction(content, self.direction)
        .width(self.width)
        .height(self.height)
        .class(self.class);
      if let Some(id) = self.id {
        scrollable = scrollable.id(id);
      }
      if FS::IS_SOME {
        scrollable = scrollable.on_scroll(move |viewport| self.on_scroll.call(viewport).unwrap());
      }
      scrollable.into()
    })
  }
}
