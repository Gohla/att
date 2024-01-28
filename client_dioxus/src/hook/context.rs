use std::any::type_name;

use dioxus::core::ScopeState;

/// Extension trait for using context hooks.
pub trait UseContextExt<T> {
  /// Uses a `context` provider hook on the component of `self`, providing `context` to the component and its children.
  fn use_context_provider(&self, context: &T) -> &T;

  /// Uses a context hook on the component of `self`, returning `Some(&context)` if a context of `T` exists in this
  /// component or a parent component, `None` otherwise.
  fn use_context(&self) -> Option<&T>;
  /// Uses a context hook on the component of `self`, returning `&context` if a context of `T` exists in this component
  /// or a parent component, panicking otherwise.
  #[inline]
  fn use_context_unwrap(&self) -> &T {
    self.use_context()
      .unwrap_or_else(|| panic!("expected context of type `{}`, but it does not exist in this component or a parent component", type_name::<T>()))
  }
}
impl<T: Clone + 'static> UseContextExt<T> for ScopeState {
  #[inline]
  fn use_context_provider(&self, value: &T) -> &T { self.use_hook(|| self.provide_context(value.clone())) }
  #[inline]
  fn use_context(&self) -> Option<&T> { self.use_hook(|| self.consume_context()).as_ref() }
}
