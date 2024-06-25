use std::future::Future;

use iced::Task;

use att_core::util::maybe_send::MaybeSend;

/// Update received from components.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Update<A = (), C = ()> {
  action: A,
  task: C,
}

impl<A: Default, M> Default for Update<A, Task<M>> {
  fn default() -> Self {
    Self::new(A::default(), Task::none())
  }
}
impl<A: Default, M: 'static> From<Task<M>> for Update<A, Task<M>> {
  fn from(task: Task<M>) -> Self {
    Self::from_task(task)
  }
}

impl<A, C> Update<A, C> {
  pub fn new(action: A, task: C) -> Self {
    Self { action, task }
  }
}
impl<A: Default, M> Update<A, Task<M>> {
  pub fn empty() -> Self {
    Self::new(A::default(), Task::none())
  }
}
impl<A, M> Update<A, Task<M>> {
  pub fn from_action(action: impl Into<A>) -> Self {
    Self::new(action.into(), Task::none())
  }
}
impl<A: Default, M: 'static> Update<A, Task<M>> {
  pub fn from_task(task: impl Into<Task<M>>) -> Self {
    Self::new(A::default(), task.into())
  }

  pub fn perform<T: MaybeSend + 'static>(
    future: impl Future<Output=T> + MaybeSend + 'static,
    f: impl FnOnce(T) -> M + MaybeSend + 'static,
  ) -> Self {
    Self::from_task(perform(future, f))
  }
}

impl<A, C> Update<A, C> {
  pub fn into_action_task(self) -> (A, C) { (self.action, self.task) }

  pub fn action(&self) -> &A { &self.action }
  pub fn into_action(self) -> A { self.action }
  pub fn take_action(self) -> (A, Update<(), C>) {
    (self.action, Update::new((), self.task))
  }
  pub fn discard_action(self) -> Update<(), C> {
    Update::new((), self.task)
  }
  pub fn map_action<AA>(self, f: impl Fn(A) -> AA) -> Update<AA, C> {
    Update::new(f(self.action), self.task)
  }

  pub fn task(&self) -> &C { &self.task }
  pub fn into_task(self) -> C { self.task }
  pub fn take_task(self) -> (C, Update<A, ()>) {
    (self.task, Update::new(self.action, ()))
  }
  pub fn discard_task(self) -> Update<A, ()> {
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
impl<A, M> Update<A, Task<M>> {
  pub fn map_task<MM>(self, f: impl FnMut(M) -> MM + 'static + MaybeSend) -> Update<A, Task<MM>> where
    M: MaybeSend + 'static,
    MM: MaybeSend + 'static
  {
    Update::new(self.action, self.task.map(f))
  }
}


pub trait Perform<T, M> {
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M>;
}
impl<T, M, F> Perform<T, M> for F where
  T: 'static,
  M: 'static,
  F: Future<Output=T> + MaybeSend + 'static,
{
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> {
    perform(self, f)
  }
}

pub trait PerformResult<T, M> {
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M>;
}
impl<T, E, M, F> PerformResult<T, M> for F where
  Result<T, E>: MaybeSend + 'static,
  M: Default + 'static,
  F: Future<Output=Result<T, E>> + MaybeSend + 'static,
{
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> {
    perform(self, |r| r.map(f).unwrap_or_default())
  }
}

pub trait PerformInto<T, M> {
  fn perform_into(self, into_message: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M>;
}
impl<T, M, F> PerformInto<T, M> for F where
  T: From<F::Output> + MaybeSend + 'static,
  M: 'static,
  F: Future + MaybeSend + 'static,
{
  fn perform_into(self, into_message: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> {
    perform(self, |future| into_message(future.into()))
  }
}
pub trait OptPerformInto<T, M> {
  fn opt_perform_into(self, into_message: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M>;
}
impl<T, M, F> OptPerformInto<T, M> for Option<F> where
  T: From<F::Output>,
  M: 'static,
  F: Future + MaybeSend + 'static,
{
  fn opt_perform_into(self, into_message: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> {
    match self {
      None => Task::none(),
      Some(future) => perform(future, |output| into_message(output.into())),
    }
  }
}

fn perform<O: 'static, T: 'static>(
  future: impl Future<Output=O> + MaybeSend + 'static,
  f: impl FnOnce(O) -> T + MaybeSend + 'static,
) -> Task<T> {
  use iced::futures::FutureExt;
  Task::future(future.map(f))
}
