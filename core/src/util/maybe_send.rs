use std::future::Future;

/// A trait alias that enforces `Send` only on native platforms.
pub use maybe_send::MaybeSend;

/// A future that implements `Send` only on native platforms.
pub trait MaybeSendFuture<'a>: Future {
  type Boxed: Future<Output=Self::Output>;
  fn boxed_maybe_send(self) -> Self::Boxed;
}

/// An optional future that implements `Send` only on native platforms.
pub trait MaybeSendOptFuture<'a> {
  type Output;
  type Boxed: Future<Output=Self::Output>;
  fn opt_boxed_maybe_send(self) -> Option<Self::Boxed>;
}

#[cfg(not(target_arch = "wasm32"))]
mod maybe_send {
  use std::future::Future;
  use std::pin::Pin;

  use crate::util::maybe_send::MaybeSendFuture;

  pub trait MaybeSend: Send {}

  impl<T> MaybeSend for T where T: Send {}

  impl<'a, F: Future + Send + 'a> super::MaybeSendFuture<'a> for F {
    type Boxed = Pin<Box<dyn Future<Output=F::Output> + Send + 'a>>;
    #[inline]
    fn boxed_maybe_send(self) -> Self::Boxed { Box::pin(self) }
  }

  impl<'a, F: Future + Send + 'a> super::MaybeSendOptFuture<'a> for Option<F> {
    type Output = F::Output;
    type Boxed = Pin<Box<dyn Future<Output=F::Output> + Send + 'a>>;
    #[inline]
    fn opt_boxed_maybe_send(self) -> Option<Self::Boxed> {
      self.map(|fut| fut.boxed_maybe_send())
    }
  }
}

#[cfg(target_arch = "wasm32")]
mod maybe_send {
  use std::future::Future;
  use std::pin::Pin;

  pub trait MaybeSend {}

  impl<T> MaybeSend for T {}

  impl<'a, F: Future + 'a> super::MaybeSendFuture<'a> for F {
    type Boxed = Pin<Box<dyn Future<Output=F::Output> + 'a>>;
    #[inline]
    fn boxed_maybe_send(self) -> Self::Boxed { Box::pin(self) }
  }
}
