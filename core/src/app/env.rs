pub use dotenvy_macro::dotenv;

pub fn load_dotenv_into_env() {
  let _ = dotenvy::dotenv(); // Ignore error ok: .env file is not required.
}

#[macro_export]
macro_rules! run_or_compile_time_env {
  ($env:literal) => {{
    std::env::var($env).unwrap_or_else(|_| $crate::app::env::dotenv!($env).to_string())
  }};
}
pub use run_or_compile_time_env;
