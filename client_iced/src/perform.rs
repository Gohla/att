use std::future::Future;

use iced::Task;

use att_core::util::maybe_send::MaybeSend;

/// Perform extension trait, implemented on [Future].
pub trait PerformExt: Future + MaybeSend + Sized + 'static {
  /// Perform this future, turning it into a [Task]. When the future completes, apply `f` to its output, creating a
  /// message of type [M].
  #[inline]
  fn perform<M: 'static>(self, f: impl FnOnce(Self::Output) -> M + MaybeSend + 'static) -> Task<M> {
    use iced::futures::FutureExt;
    Task::future(self.map(f))
  }

  /// Perform this future, turning it into a [Task]. When the future completes, turn it [into](Into) a value of type
  /// [T], then apply `f` to that value, creating a message of type [M].
  #[inline]
  fn perform_into<T: 'static, M: 'static>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> where
    Self::Output: Into<T>,
  {
    self.perform(|output| f(output.into()))
  }

  /// Perform this future, turning it into a [Task]. When the future completes
  ///
  /// - If the future's output is `Ok(v)`: create message `f(v)` of type [M].
  /// - If the future's output is `Err(_)`: create message `M::default()`.
  #[inline]
  fn perform_or_default<T, E, M>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> where
    Self: Future<Output=Result<T, E>>,
    Result<T, E>: MaybeSend + 'static,
    M: Default + 'static,
  {
    self.perform(|r| r.map(f).unwrap_or_default())
  }
}
impl<F: Future + MaybeSend + Sized + 'static> PerformExt for F {}


/// Option perform extension trait, implemented on [Option<impl Future>].
pub trait OptionPerformExt {
  /// The type of value produced when the future completes.
  type Output;

  /// Perform this optional future, turning it into a [Task]:
  ///
  /// - If this future is `Some(future)`, return [PerformExt::perform_into].
  /// - If this future is `None`, return [Task::none].
  fn opt_perform_into<T: 'static, M: 'static>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> where
    Self::Output: Into<T>;
}
impl<F: Future + MaybeSend + 'static> OptionPerformExt for Option<F> {
  type Output = F::Output;

  #[inline]
  fn opt_perform_into<T: 'static, M: 'static>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Task<M> where
    Self::Output: Into<T>,
  {
    match self {
      None => Task::none(),
      Some(future) => future.perform_into(f),
    }
  }
}
