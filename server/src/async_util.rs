use tokio::sync::{mpsc, oneshot};

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
