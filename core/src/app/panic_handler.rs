pub fn install_panic_handler() {
  #[cfg(target_arch = "wasm32")]
  std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
