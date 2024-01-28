use std::sync::Arc;

use dioxus::core::ScopeState;

/// Hook that provides (im)mutable access to a value, triggering an update of the component this hook belongs to when
/// the value is mutated.
pub struct UseValue<T> {
  value: T,
  update: Arc<dyn Fn()>,
}

/// Extension trait for using [value hooks](UseValue).
pub trait UseValueExt<T> {
  /// Uses a [value hook](UseValue) on the component of `self` with an initial `value`.
  fn use_value(&self, value: T) -> &mut UseValue<T>;
  /// Uses a [value hook](UseValue) on the component of `self` with a [default](Default) initial value.
  #[inline]
  fn use_value_default(&self) -> &mut UseValue<T> where T: Default { self.use_value(T::default()) }
}
impl<T: 'static> UseValueExt<T> for ScopeState {
  #[inline]
  fn use_value(&self, value: T) -> &mut UseValue<T> {
    self.use_hook(move || UseValue { value, update: self.schedule_update() })
  }
}

impl<T> UseValue<T> {
  /// Gets the immutable value.
  #[inline]
  pub fn get(&self) -> &T { &self.value }
  /// Gets the mutable value. Triggers update of the component this hook belongs to.
  #[inline]
  pub fn get_mut(&mut self) -> &mut T {
    (self.update)();
    &mut self.value
  }
}
impl<T> AsRef<T> for UseValue<T> {
  #[inline]
  fn as_ref(&self) -> &T { self.get() }
}
// Note: AsMut is not implemented, because `get_mut` is not cheap, as it runs an update function.
