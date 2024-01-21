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
