use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use att_core::util::http_status_code::AsStatusCode;

/// A [`Result`] that implements [`IntoResponse`] by turning both `Ok(v)` and `Err(e)` into a [`Json`] response.
///
/// The value type [`T`] must implement [`Serialize`].
/// The error type [`E`] must implement [`Serialize`] and [`AsStatusCode`].
pub type JsonResult<T, E> = Result<JsonOk<T>, JsonErr<E>>;

/// A [`Result`] that implements [`IntoResponse`] by turning `Ok(v)` into a [`Json`] response, and `Err(e)` into a
/// [`StatusCode`](axum::http::StatusCode) response.
///
/// The value type [`T`] must implement [`Serialize`].
/// The error type [`E`] must implement [`AsStatusCode`].
pub type JsonStatus<T, E> = Result<JsonOk<T>, StatusErr<E>>;


/// A value that implements [`IntoResponse`] by turning it into a `Json(Ok(v))` response.
///
/// The value type [`T`] must implement [`Serialize`].
pub struct JsonOk<T>(pub T);

impl<T> JsonOk<T> {
  pub fn new(err: T) -> Self { Self(err) }
}

impl<T> From<T> for JsonOk<T> {
  fn from(err: T) -> Self { Self::new(err) }
}

impl<T: Serialize> IntoResponse for JsonOk<T> {
  fn into_response(self) -> Response {
    Json(Ok::<T, ()>(self.0)).into_response()
  }
}


/// An error that implements [`IntoResponse`] by turning it into a `Json(Err(e))` response with a status code.
///
/// The error type [`E`] must implement [`Serialize`] and [`AsStatusCode`].
pub struct JsonErr<E>(pub E);

impl<E> JsonErr<E> {
  pub fn new(err: E) -> Self { Self(err) }
}

impl<E> From<E> for JsonErr<E> {
  fn from(err: E) -> Self { Self::new(err) }
}

impl<E: Serialize + AsStatusCode> IntoResponse for JsonErr<E> {
  fn into_response(self) -> Response {
    (self.0.as_status_code(), Json(self.0)).into_response()
  }
}


/// An error that implements [`IntoResponse`] by turning it into a status code response.
///
/// The error type [`E`] must and [`AsStatusCode`].
pub struct StatusErr<E>(pub E);

impl<E> StatusErr<E> {
  pub fn new(err: E) -> Self { Self(err) }
}

impl<E> From<E> for StatusErr<E> {
  fn from(err: E) -> Self { Self::new(err) }
}

impl<E: AsStatusCode> IntoResponse for StatusErr<E> {
  fn into_response(self) -> Response {
    self.0.as_status_code().into_response()
  }
}
