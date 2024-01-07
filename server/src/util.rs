use std::error::Error;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct F(StatusCode);
impl F {
  pub fn unauthorized() -> Self { Self(StatusCode::UNAUTHORIZED) }
  pub fn forbidden() -> Self { Self(StatusCode::FORBIDDEN) }
  pub fn error() -> Self { Self(StatusCode::INTERNAL_SERVER_ERROR) }
}
impl IntoResponse for F {
  fn into_response(self) -> Response {
    self.0.into_response()
  }
}
impl<E: Error> From<E> for F {
  fn from(_: E) -> Self { Self::error() }
}
