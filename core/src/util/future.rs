use std::future::Future;

use iced::futures::FutureExt;

pub trait OptFutureExt {
  type Output;

  fn opt_map<U>(self, f: impl FnOnce(Self::Output) -> U) -> Option<impl Future<Output=U>>;
  fn opt_map_into<U>(self) -> Option<impl Future<Output=U>> where
    Self::Output: Into<U>;
}

impl<F: Future> OptFutureExt for Option<F> {
  type Output = F::Output;

  #[inline]
  fn opt_map<U>(self, f: impl FnOnce(Self::Output) -> U) -> Option<impl Future<Output=U>> {
    self.map(|fut| fut.map(f))
  }

  #[inline]
  fn opt_map_into<U>(self) -> Option<impl Future<Output=U>> where
    Self::Output: Into<U>
  {
    self.map(|fut| fut.map_into())
  }
}
