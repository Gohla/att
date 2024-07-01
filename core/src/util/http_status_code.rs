pub use http::StatusCode;

pub trait AsStatusCode {
  fn as_status_code(&self) -> StatusCode;
}

impl AsStatusCode for StatusCode {
  fn as_status_code(&self) -> StatusCode { *self }
}
