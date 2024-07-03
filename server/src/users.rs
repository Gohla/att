use std::fmt::Debug;
use std::ops::Deref;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{self, SaltString};
use axum::{async_trait, Json, Router};
use axum_login::{AuthnBackend, AuthUser, UserId};
use rand_core::OsRng;
use thiserror::Error;
use tracing::instrument;

use att_core::users::{AuthError, UserCredentials};
use att_server_db::{DbError, DbPool};
use att_server_db::users::{NewUser, User, UsersDb};

use crate::util::JsonResult;

#[derive(Clone)]
pub struct Users {
  argon2: Argon2<'static>,
  db_pool: DbPool<UsersDb>,
}

impl Users {
  pub fn new(argon2: Argon2<'static>, db_pool: DbPool) -> Self {
    Self { argon2, db_pool: db_pool.with() }
  }

  pub fn from_db_pool(db_pool: DbPool) -> Self {
    Self::new(Argon2::default(), db_pool)
  }

  #[inline]
  pub fn db_pool(&self) -> &DbPool<UsersDb> { &self.db_pool }
}


#[derive(Debug, Error)]
pub enum InternalError {
  #[error("Parsing hash or hashing password failed: {0}")]
  HashPassword(#[from] password_hash::Error),
  #[error("Database operation failed: {0}")]
  Database(#[from] DbError),
}

impl Users {
  #[instrument(skip_all, err)]
  pub async fn ensure_default_user_exists(&self) -> Result<bool, InternalError> {
    let user_credentials = UserCredentials::default();
    let password_hash = self.hash_password(user_credentials.password.as_bytes())?;
    let created = self.db_pool.interact(move |conn| {
      let created = if conn.get_by_name(&user_credentials.name)?.is_none() {
        let user = conn.insert(NewUser { name: user_credentials.name, password_hash })?;
        user.is_some()
      } else {
        false
      };
      Ok::<_, InternalError>(created)
    }).await??;
    Ok(created)
  }

  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  async fn authenticate_user(&self, user_credentials: UserCredentials) -> Result<Option<User>, InternalError> {
    let user = self.db_pool
      .query(move |db| db.get_by_name(&user_credentials.name))
      .await?;
    let user = if let Some(user) = user {
      let parsed_hash = PasswordHash::new(&user.password_hash)?;
      match self.argon2.verify_password(user_credentials.password.as_bytes(), &parsed_hash) {
        Ok(()) => Some(user),
        Err(password_hash::Error::Password) => None,
        Err(e) => Err(e)?,
      }
    } else {
      None
    };
    Ok(user)
  }

  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  async fn create_user(&self, user_credentials: UserCredentials) -> Result<Option<User>, InternalError> {
    let password_hash = self.hash_password(user_credentials.password.as_bytes())?;
    let user = self.db_pool
      .interact(|conn| conn.insert(NewUser { name: user_credentials.name, password_hash }))
      .await??;
    Ok(user)
  }


  fn hash_password(&self, password: &[u8]) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = self.argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
  }
}


// Authentication

#[derive(Clone, Debug)]
pub struct LoginUser(pub User);
impl Deref for LoginUser {
  type Target = User;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.0 }
}

impl AuthUser for LoginUser {
  type Id = i32;
  fn id(&self) -> Self::Id { self.id }

  fn session_auth_hash(&self) -> &[u8] { self.password_hash.as_bytes() }
}

#[async_trait]
impl AuthnBackend for Users {
  type User = LoginUser;
  type Credentials = UserCredentials;
  type Error = InternalError;

  async fn authenticate(&self, credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
    let user = self.authenticate_user(credentials).await?.map(LoginUser);
    Ok(user)
  }

  async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    let user_id = *user_id;
    let user = self.db_pool.query(move |db| db.find(user_id)).await?.map(LoginUser);
    Ok(user)
  }
}

pub type AuthSession = axum_login::AuthSession<Users>;


// Router

pub fn router() -> Router<()> {
  use axum::routing::post;
  Router::new()
    .route("/login", post(login).delete(logout))
}

async fn login(mut auth_session: AuthSession, Json(credentials): Json<UserCredentials>) -> JsonResult<(), AuthError> {
  let user = auth_session.authenticate(credentials.clone()).await
    .map_err(|_| AuthError::Internal)?
    .ok_or(AuthError::IncorrectUserNameOrPassword)?;
  auth_session.login(&user).await
    .map_err(|_| AuthError::Internal)?;
  Ok(().into())
}

async fn logout(mut auth_session: AuthSession) -> JsonResult<(), AuthError> {
  auth_session.logout().await
    .map_err(|_| AuthError::Internal)?;
  Ok(().into())
}
