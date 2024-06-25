pub trait IsSome {
  const IS_SOME: bool;
}
pub trait TOption<T>: IsSome {
  fn unwrap(self) -> T;
}
pub trait TOptionFn<'a, I, O>: IsSome {
  fn call(&self, input: I) -> impl TOption<O>;
}

pub struct TNone;
impl IsSome for TNone {
  const IS_SOME: bool = false;
}
impl<T> TOption<T> for TNone {
  #[inline]
  fn unwrap(self) -> T { panic!("called `TOption::unwrap()` on a `TNone` value"); }
}
impl<'a, I, O> TOptionFn<'a, I, O> for TNone {
  #[inline]
  fn call(&self, _input: I) -> impl TOption<O> { TNone }
}

pub struct TSome<T>(pub T);
impl<T> IsSome for TSome<T> {
  const IS_SOME: bool = true;
}
impl<T> TOption<T> for TSome<T> {
  #[inline]
  fn unwrap(self) -> T { self.0 }
}
impl<'a, I, O, F: Fn(I) -> O + 'a> TOptionFn<'a, I, O> for TSome<F> {
  #[inline]
  fn call(&self, input: I) -> impl TOption<O> { TSome(self.0(input)) }
}
