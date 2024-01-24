pub use http::StatusCode;

pub trait AsStatusCode {
  fn as_status_code(&self) -> StatusCode;
}
