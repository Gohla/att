use iced::{Command, Element};
use iced::advanced::Widget;
use iced::widget::Button;
use iced_futures::MaybeSend;

/// Update received from components.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Update<A = (), C = ()> {
  action: A,
  command: C,
}
impl<A, C> Update<A, C> {
  pub fn new(action: A, command: C) -> Self { Self { action, command } }
}
impl<A> Update<A, ()> {
  pub fn from_action(action: impl Into<A>) -> Self { Self::new(action.into(), ()) }
}
impl<C> Update<(), C> {
  pub fn from_command(command: impl Into<C>) -> Self { Self::new((), command.into()) }
}
impl<A, C> Update<A, C> {
  pub fn unwrap(self) -> (A, C) { (self.action, self.command) }

  pub fn action(&self) -> &A { &self.action }
  pub fn into_action(self) -> A { self.action }
  pub fn take_action(self) -> (A, Update<(), C>) {
    (self.action, Update::from_command(self.command))
  }
  pub fn discard_action(self) -> Update<(), C> {
    Update::from_command(self.command)
  }
  pub fn map_action<AA>(self, f: impl Fn(A) -> AA) -> Update<AA, C> {
    Update::new(f(self.action), self.command)
  }

  pub fn command(&self) -> &C { &self.command }
  pub fn into_command(self) -> C { self.command }
  pub fn take_command(self) -> (C, Update<A, ()>) {
    (self.command, Update::from_action(self.action))
  }
  pub fn discard_command(self) -> Update<A, ()> {
    Update::from_action(self.action)
  }
}
impl<A, C> Update<Option<A>, C> {
  pub fn inspect_action(&self, f: impl FnOnce(&A)) {
    if let Some(action) = &self.action {
      f(action)
    }
  }
}
impl<A, M> Update<A, Command<M>> {
  pub fn map_command<MM>(self, f: impl Fn(M) -> MM + 'static + MaybeSend + Sync + Clone) -> Update<A, Command<MM>> where
    M: 'static,
    MM: 'static
  {
    Update::new(self.action, self.command.map(f))
  }
}

/// Widget extensions
pub trait WidgetExt<'a, M, R> {
  fn into_element(self) -> Element<'a, M, R>;
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R>;
}
impl<'a, M: 'a, R: iced::advanced::Renderer + 'a, W: Widget<M, R> + 'a> WidgetExt<'a, M, R> for W {
  #[inline]
  fn into_element(self) -> Element<'a, M, R> {
    Element::new(self)
  }
  #[inline]
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R> {
    self.into_element().map(f)
  }
}

/// Button widget extensions
pub trait ButtonEx<'a, R> {
  fn on_press_into_element<M: 'a, F: Fn() -> M + 'a>(self, f: F) -> Element<'a, M, R>;
}
impl<'a, R> ButtonEx<'a, R> for Button<'a, (), R> where
  R: iced::advanced::Renderer + 'a,
  R::Theme: iced::widget::button::StyleSheet,
{
  fn on_press_into_element<M: 'a, F: Fn() -> M + 'a>(self, f: F) -> Element<'a, M, R> {
    self.on_press(()).map_into_element(move |_| f())
  }
}

/// Copy of column! macro, which the Rust plugin does not like due to the built-in column! macro.
#[macro_export]
macro_rules! col {
    () => (
        iced::widget::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        iced::widget::Column::with_children(vec![$(iced::Element::from($x)),+])
    );
}
