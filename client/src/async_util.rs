use std::future::Future;

use iced::Command;

pub use maybe_send::MaybeSend;

#[cfg(not(target_arch = "wasm32"))]
mod maybe_send {
  /// An extension trait that enforces `Send` only on native platforms.
  ///
  /// Useful to write cross-platform async code!
  pub trait MaybeSend: Send {}

  impl<T> MaybeSend for T where T: Send {}
}

#[cfg(target_arch = "wasm32")]
mod maybe_send {
  /// An extension trait that enforces `Send` only on native platforms.
  ///
  /// Useful to write cross-platform async code!
  pub trait MaybeSend {}

  impl<T> MaybeSend for T {}
}


pub trait PerformFutureExt<T, M> {
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M>;
}
impl<T, M, F: Future<Output=T> + MaybeSend + 'static> PerformFutureExt<T, M> for F {
  fn perform(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, |v| f(v))
  }
}

pub trait PerformResultFutureExt<T, M> {
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M>;
}
impl<T, E, M: Default, F: Future<Output=Result<T, E>> + MaybeSend + 'static> PerformResultFutureExt<T, M> for F {
  fn perform_or_default(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, |r| r.map(f).unwrap_or_default())
  }
}
