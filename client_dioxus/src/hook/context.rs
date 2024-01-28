use dioxus::core::ScopeState;

/// Extension trait for creating [value hooks](UseContext).
pub trait UseContextExt<T> {
  /// Uses a `context` provider hook on the component of `self`.
  fn use_context_provider(&self, context: &T) -> &T;
  /// Uses a context hook on the component of `self`, returning `Some(&context)` if a context of `T` exists, `None`
  /// otherwise.
  fn use_context(&self) -> Option<&T>;
  /// Uses a context hook on the component of `self`, returning `&context` if a context of `T` exists, panicking
  /// otherwise.
  #[inline]
  fn use_context_unwrap(&self) -> &T { self.use_context().unwrap() }
}
impl<T: Clone + 'static> UseContextExt<T> for ScopeState {
  #[inline]
  fn use_context_provider(&self, value: &T) -> &T { self.use_hook(|| self.provide_context(value.clone())) }
  #[inline]
  fn use_context(&self) -> Option<&T> { self.use_hook(|| self.consume_context()).as_ref() }
}
