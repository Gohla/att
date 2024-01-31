use std::future::Future;

use iced::Command;

use att_core::util::maybe_send::MaybeSend;

/// Update received from components.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Update<A = (), C = ()> {
  action: A,
  command: C,
}

impl<A: Default, M> Default for Update<A, Command<M>> {
  fn default() -> Self {
    Self::new(A::default(), Command::none())
  }
}
impl<A: Default, M> From<Command<M>> for Update<A, Command<M>> {
  fn from(command: Command<M>) -> Self {
    Self::from_command(command)
  }
}

impl<A, C> Update<A, C> {
  pub fn new(action: A, command: C) -> Self {
    Self { action, command }
  }
}
impl<A: Default, M> Update<A, Command<M>> {
  pub fn empty() -> Self {
    Self::new(A::default(), Command::none())
  }
}
impl<A, M> Update<A, Command<M>> {
  pub fn from_action(action: impl Into<A>) -> Self {
    Self::new(action.into(), Command::none())
  }
}
impl<A: Default, M> Update<A, Command<M>> {
  pub fn from_command(command: impl Into<Command<M>>) -> Self {
    Self::new(A::default(), command.into())
  }

  pub fn perform<T>(
    future: impl Future<Output=T> + 'static + MaybeSend,
    f: impl FnOnce(T) -> M + 'static + MaybeSend,
  ) -> Self {
    Self::from_command(Command::perform(future, f))
  }
}

impl<A, C> Update<A, C> {
  pub fn into_action_command(self) -> (A, C) { (self.action, self.command) }

  pub fn action(&self) -> &A { &self.action }
  pub fn into_action(self) -> A { self.action }
  pub fn take_action(self) -> (A, Update<(), C>) {
    (self.action, Update::new((), self.command))
  }
  pub fn discard_action(self) -> Update<(), C> {
    Update::new((), self.command)
  }
  pub fn map_action<AA>(self, f: impl Fn(A) -> AA) -> Update<AA, C> {
    Update::new(f(self.action), self.command)
  }

  pub fn command(&self) -> &C { &self.command }
  pub fn into_command(self) -> C { self.command }
  pub fn take_command(self) -> (C, Update<A, ()>) {
    (self.command, Update::new(self.action, ()))
  }
  pub fn discard_command(self) -> Update<A, ()> {
    Update::new(self.action, ())
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


pub trait Perform<T, M> {
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M>;
}
impl<T, M, F: Future<Output=T> + MaybeSend + 'static> Perform<T, M> for F {
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, f)
  }
}

pub trait PerformResult<T, M> {
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M>;
}
impl<T, E, M: Default, F: Future<Output=Result<T, E>> + MaybeSend + 'static> PerformResult<T, M> for F {
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, |r| r.map(f).unwrap_or_default())
  }
}

pub trait PerformInto<T, I, M> {
  fn perform_into(self, f: impl FnOnce(I) -> M + MaybeSend + 'static) -> Command<M>;
}
impl<T, I: From<T>, M, F: Future<Output=T> + MaybeSend + 'static> PerformInto<T, I, M> for F {
  fn perform_into(self, f: impl FnOnce(I) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, |t| f(t.into()))
  }
}
