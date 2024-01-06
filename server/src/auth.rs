use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use axum::{async_trait, Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_login::{AuthnBackend, AuthUser, UserId};
use serde::{Deserialize, Serialize};

use crate::data::Database;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Users {
  id_to_user: HashMap<u64, User>,
  name_to_user: HashMap<String, User>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  id: u64,
  name: String,
  password_hash: String,
}
impl Debug for User {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("User")
      .field("id", &self.id)
      .field("name", &self.name)
      .field("password_hash", &"[redacted]")
      .finish()
  }
}
impl AuthUser for User {
  type Id = u64;
  fn id(&self) -> Self::Id { self.id }
  fn session_auth_hash(&self) -> &[u8] { self.password_hash.as_bytes() }
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
  name: String,
  password_hash: String,
}
impl Debug for Credentials {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("User")
      .field("name", &self.name)
      .field("password_hash", &"[redacted]")
      .finish()
  }
}

#[derive(Clone)]
pub struct Authenticator(Database);
impl Authenticator {
  pub fn new(database: Database) -> Self { Self(database) }
}
#[async_trait]
impl AuthnBackend for Authenticator {
  type User = User;
  type Credentials = Credentials;
  type Error = std::convert::Infallible;
  async fn authenticate(&self, credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
    let user = self.0.read().await.users.name_to_user.get(&credentials.name)
      .filter(|u| u.password_hash == credentials.password_hash)
      .cloned();
    Ok(user)
  }
  async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    Ok(self.0.read().await.users.id_to_user.get(user_id).cloned())
  }
}
pub type AuthSession = axum_login::AuthSession<Authenticator>;

pub fn router() -> Router<()> {
  use axum::routing::post;
  Router::new()
    .route("/login", post(login).delete(logout))
}
async fn login(mut auth_session: AuthSession, Json(credentials): Json<Credentials>) -> impl IntoResponse {
  let user = match auth_session.authenticate(credentials.clone()).await {
    Ok(Some(user)) => user,
    Ok(None) => {
      // TODO: return actual error
      return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };

  if auth_session.login(&user).await.is_err() {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  }

  StatusCode::OK.into_response()
}
async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
  match auth_session.logout().await {
    Ok(_) => StatusCode::OK.into_response(),
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}
