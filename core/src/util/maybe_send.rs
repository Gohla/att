use std::future::Future;

pub use maybe_send::MaybeSend;

/// An extension trait for boxing futures, where boxed futures implement `Send` only on native platforms.
pub trait MaybeSendFuture<'a>: Future {
  type Boxed: Future<Output=Self::Output>;
  fn boxed_maybe_send(self) -> Self::Boxed;
}

#[cfg(not(target_arch = "wasm32"))]
mod maybe_send {
  use std::future::Future;
  use std::pin::Pin;

  /// A trait alias that enforces `Send` only on native platforms.
  pub trait MaybeSend: Send {}

  impl<T> MaybeSend for T where T: Send {}

  impl<'a, F: Future + Send + 'a> super::MaybeSendFuture<'a> for F {
    type Boxed = Pin<Box<dyn Future<Output=F::Output> + Send + 'a>>;
    #[inline]
    fn boxed_maybe_send(self) -> Self::Boxed { Box::pin(self) }
  }
}

#[cfg(target_arch = "wasm32")]
mod maybe_send {
  use std::future::Future;
  use std::pin::Pin;

  /// A trait alias that enforces `Send` only on native platforms.
  pub trait MaybeSend {}

  impl<T> MaybeSend for T {}

  impl<'a, F: Future + 'a> super::MaybeSendFuture<'a> for F {
    type Boxed = Pin<Box<dyn Future<Output=F::Output> + 'a>>;
    #[inline]
    fn boxed_maybe_send(self) -> Self::Boxed { Box::pin(self) }
  }
}
