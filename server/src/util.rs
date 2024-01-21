use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use att_core::util::status_code::AsStatusCode;

pub struct JsonResult<T, E>(pub Result<T, E>);

impl<T, E> JsonResult<T, E> {
  #[inline]
  pub fn new(result: Result<T, E>) -> Self { Self(result) }
}
impl<T, E> From<Result<T, E>> for JsonResult<T, E> {
  #[inline]
  fn from(result: Result<T, E>) -> Self { Self::new(result) }
}

impl<T: Serialize, E: Serialize + AsStatusCode> IntoResponse for JsonResult<T, E> {
  #[inline]
  fn into_response(self) -> Response {
    match self.0 {
      r @ Ok(_) => Json(r).into_response(),
      ref r @ Err(ref e) => (e.as_status_code(), Json(r)).into_response(),
    }
  }
}
