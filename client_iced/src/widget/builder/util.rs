pub trait TOption<T>: IsSome {
  fn map<O>(self, map_fn: impl FnOnce(T) -> O) -> impl TOption<O>;
  fn map_or<O>(self, some_fn: impl FnOnce(T) -> O, none_output: O) -> O;
  fn map_or_else<O>(self, some_fn: impl FnOnce(T) -> O, none_fn: impl FnOnce() -> O) -> O;

  fn unwrap(self) -> T;
  fn unwrap_or(self, none_output: T) -> T;
  fn unwrap_or_else(self, none_fn: impl FnOnce() -> T) -> T;

  fn if_some(self, some_fn: impl FnMut(T));

  fn into_option(self) -> Option<T>;
}
pub trait IsSome {
  const IS_SOME: bool;
}
pub trait TOptionFn<'a, I, O> {
  fn call(&self, input: I) -> impl TOption<O>;
}

pub struct TNone;
impl<T> TOption<T> for TNone {
  #[inline]
  fn map<O>(self, _map_fn: impl FnOnce(T) -> O) -> impl TOption<O> { Self }
  #[inline]
  fn map_or<O>(self, _some_fn: impl FnOnce(T) -> O, none_output: O) -> O { none_output }
  #[inline]
  fn map_or_else<O>(self, _some_fn: impl FnOnce(T) -> O, none_fn: impl FnOnce() -> O) -> O { none_fn() }

  #[inline]
  fn unwrap(self) -> T { panic!("called `TOption::unwrap()` on a `TNone` value") }
  #[inline]
  fn unwrap_or(self, none_output: T) -> T { none_output }
  #[inline]
  fn unwrap_or_else(self, none_fn: impl FnOnce() -> T) -> T { none_fn() }

  #[inline]
  fn if_some(self, _some_fn: impl FnMut(T)) {}

  #[inline]
  fn into_option(self) -> Option<T> { None }
}
impl IsSome for TNone {
  const IS_SOME: bool = false;
}

impl<'a, I, O> TOptionFn<'a, I, O> for TNone {
  #[inline]
  fn call(&self, _input: I) -> impl TOption<O> { TNone }
}

pub struct TSome<T>(pub T);
impl<T> TOption<T> for TSome<T> {
  #[inline]
  fn map<O>(self, map_fn: impl FnOnce(T) -> O) -> impl TOption<O> { TSome(map_fn(self.0)) }
  #[inline]
  fn map_or<O>(self, some_fn: impl FnOnce(T) -> O, _none_output: O) -> O { some_fn(self.0) }
  #[inline]
  fn map_or_else<O>(self, some_fn: impl FnOnce(T) -> O, _none_fn: impl FnOnce() -> O) -> O { some_fn(self.0) }

  #[inline]
  fn unwrap(self) -> T { self.0 }
  #[inline]
  fn unwrap_or(self, _none_output: T) -> T { self.0 }
  #[inline]
  fn unwrap_or_else(self, _none_fn: impl FnOnce() -> T) -> T { self.0 }

  #[inline]
  fn if_some(self, mut some_fn: impl FnMut(T)) { some_fn(self.0) }

  #[inline]
  fn into_option(self) -> Option<T> { Some(self.0) }
}
impl<'a, I, O, F: Fn(I) -> O + 'a> TOptionFn<'a, I, O> for TSome<F> {
  #[inline]
  fn call(&self, input: I) -> impl TOption<O> { TSome(self.0(input)) }
}
impl<T> IsSome for TSome<T> {
  const IS_SOME: bool = true;
}
