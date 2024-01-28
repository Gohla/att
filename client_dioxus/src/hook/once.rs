use dioxus::core::ScopeState;

/// Extension trait for using once hooks.
pub trait UseOnceExt<T> {
  /// Uses a once hook on the component of `self`, calling `f` once and returning the value it produced.
  fn use_once(&self, f: impl FnOnce() -> T) -> &mut T;
}
impl<T: 'static> UseOnceExt<T> for ScopeState {
  fn use_once(&self, f: impl FnOnce() -> T) -> &mut T {
    self.use_hook(f)
  }
}
