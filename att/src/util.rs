use iced::{Command, Element};
use iced::widget::Button;
use iced_core::Widget;
use iced_futures::MaybeSend;

/// Update received from components.
pub struct Update<A, M = ()> {
  action: Option<A>,
  command: Command<M>,
}
impl<A, M> Update<A, M> {
  pub fn new(action: Option<A>, command: Command<M>) -> Self { Self { command, action } }
  pub fn from_action(action: A) -> Self { Self::new(Some(action), Command::none()) }
  pub fn from_command(command: Command<M>) -> Self { Self::new(None, command) }
  pub fn none() -> Self { Self::new(None, Command::none()) }

  pub fn unwrap(self) -> (Option<A>, Command<M>) { (self.action, self.command) }

  pub fn action(&self) -> Option<&A> { self.action.as_ref() }
  pub fn inspect_action(&self, f: impl FnOnce(&A)) {
    if let Some(action) = &self.action {
      f(action)
    }
  }
  pub fn into_action(self) -> Option<A> { self.action }
  pub fn take_action(self) -> (Option<A>, Update<(), M>) {
    (self.action, Update::from_command(self.command))
  }
  pub fn discard_action(self) -> Update<(), M> {
    Update::from_command(self.command)
  }
  pub fn map_action<AA>(self, f: impl Fn(A) -> AA) -> Update<AA, M> {
    Update::new(self.action.map(f), self.command)
  }

  pub fn command(&self) -> &Command<M> { &self.command }
  pub fn into_command(self) -> Command<M> { self.command }
  pub fn take_command(self) -> (Command<M>, Update<A, ()>) {
    (self.command, Update::new(self.action, Command::none()))
  }
  pub fn discard_command(self) -> Update<A, ()> {
    Update::new(self.action, Command::none())
  }
  pub fn map_command<MM>(self, f: impl Fn(M) -> MM + 'static + MaybeSend + Sync + Clone) -> Update<A, MM> where
    M: 'static,
    MM: 'static
  {
    Update::new(self.action, self.command.map(f))
  }
}
impl<A, M> Default for Update<A, M> {
  fn default() -> Self { Self::none() }
}

/// Widget extensions
pub trait WidgetExt<'a, M, R> {
  fn into_element(self) -> Element<'a, M, R>;
  fn map_into_element<MM: 'a, F: Fn(M) -> MM + 'a>(self, f: F) -> Element<'a, MM, R>;
}
impl<'a, M: 'a, R: iced_core::Renderer + 'a, W: Widget<M, R> + 'a> WidgetExt<'a, M, R> for W {
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
  R: iced_core::Renderer + 'a,
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
        iced::widget::Column::with_children(vec![$(iced_core::Element::from($x)),+])
    );
}
