use std::future::Future;

use iced::Command;
use tokio::sync::{mpsc, oneshot};

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


#[derive(Debug, thiserror::Error)]
pub enum AsyncError {
  #[error("Failed to send request; receiver was closed")]
  Tx,
  #[error("Failed to receive response; sender was closed")]
  Rx,
}
impl<T> From<mpsc::error::SendError<T>> for AsyncError {
  fn from(_: mpsc::error::SendError<T>) -> Self { Self::Tx }
}
impl From<oneshot::error::RecvError> for AsyncError {
  fn from(_: oneshot::error::RecvError) -> Self { Self::Rx }
}

pub trait PerformFutureExt<T> {
  fn perform<M: Default>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M>;
  fn perform_ignore<M: Default>(self) -> Command<M>;
}
impl<T, F: Future<Output=Result<T, AsyncError>> + MaybeSend + 'static> PerformFutureExt<T> for F {
  fn perform<M: Default>(self, f: impl FnOnce(T) -> M + MaybeSend + 'static) -> Command<M> {
    Command::perform(self, |r: Result<T, AsyncError>| r.map(f).unwrap_or_default())
  }
  fn perform_ignore<M: Default>(self) -> Command<M> {
    Command::perform(self, |_| M::default())
  }
}
