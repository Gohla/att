use std::fmt::{self, Debug, Formatter};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{self, SaltString};
use axum::{async_trait, Json, Router};
use axum_login::{AuthnBackend, AuthUser, UserId};
use diesel::insert_into;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::PoolError;
use diesel_async::RunQueryDsl;
use rand_core::OsRng;
use thiserror::Error;
use tracing::instrument;

use att_core::users::{AuthError, UserCredentials};

use crate::data::DbPool;
use crate::util::JsonResult;

#[derive(Clone, Queryable, Selectable, Identifiable, AsChangeset, Insertable)]
#[diesel(table_name = att_core::schema::users, check_for_backend(diesel::pg::Pg))]
pub struct User {
  pub id: i32,
  pub name: String,
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


// Implementation

#[derive(Clone)]
pub struct Users {
  argon2: Argon2<'static>,
  db_pool: DbPool,
}

#[derive(Debug, Error)]
pub enum UsersError {
  #[error("Failed to create database connection from pool")]
  DbConnection(#[from] PoolError),
  #[error("User query failed")]
  UserQuery(#[from] diesel::result::Error),
  #[error("Parsing hash or hashing password failed")]
  HashPassword(#[from] password_hash::Error),
}

impl Users {
  pub fn new(argon2: Argon2<'static>, db_pool: DbPool) -> Self {
    Self { argon2, db_pool }
  }

  pub fn from_db_pool(db_pool: DbPool) -> Self {
    Self::new(Argon2::default(), db_pool)
  }

  // #[instrument(skip_all, err)]
  // pub fn ensure_default_user_exists(&self, data: &mut UsersData) -> Result<bool, password_hash::Error> {
  //   let user_credentials = UserCredentials::default();
  //   let created = if !data.contains_user_by_name(&user_credentials.name) {
  //     self.create_user(data, user_credentials)?
  //   } else {
  //     false
  //   };
  //   Ok(created)
  // }

  #[instrument(skip(self), err)]
  async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>, UsersError> {
    use att_core::schema::users::dsl::*;
    let mut conn = self.db_pool.get().await?;
    let user = users
      .find(user_id)
      .select(User::as_select())
      .first(&mut conn).await
      .optional()?;
    Ok(user)
  }

  #[instrument(skip_all, fields(user_credentials.name = user_credentials.name), err)]
  async fn authenticate_user(&self, user_credentials: &UserCredentials) -> Result<Option<User>, UsersError> {
    let user = {
      use att_core::schema::users::dsl::*;
      let mut conn = self.db_pool.get().await?;
      users
        .filter(name.eq(&user_credentials.name))
        .select(User::as_select())
        .first(&mut conn).await
        .optional()?
    };

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
  async fn create_user(&self, user_credentials: UserCredentials) -> Result<Option<User>, UsersError> {
    #[derive(Insertable)]
    #[diesel(table_name = att_core::schema::users, check_for_backend(diesel::pg::Pg))]
    struct NewUser<'a> {
      name: &'a str,
      password_hash: &'a str,
    }

    let password_hash = self.hash_password(user_credentials.password.as_bytes())?;
    let new_user = NewUser { name: &user_credentials.name, password_hash: &password_hash };

    let user = {
      use att_core::schema::users::dsl::*;
      let mut conn = self.db_pool.get().await?;
      insert_into(users)
        .values(&new_user)
        .get_result(&mut conn).await
        .optional()?
    };

    Ok(user)
  }


  fn hash_password(&self, password: &[u8]) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = self.argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
  }
}


// Authentication

impl AuthUser for User {
  type Id = i32;
  fn id(&self) -> Self::Id { self.id }

  fn session_auth_hash(&self) -> &[u8] { self.password_hash.as_bytes() }
}

#[async_trait]
impl AuthnBackend for Users {
  type User = User;
  type Credentials = UserCredentials;
  type Error = UsersError;

  async fn authenticate(&self, credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
    let user = self.authenticate_user(&credentials).await?;
    Ok(user)
  }

  async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    let user = self.get_user_by_id(*user_id).await?;
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
  async move {
    let user = auth_session.authenticate(credentials.clone()).await
      .map_err(|_| AuthError::Internal)?
      .ok_or(AuthError::IncorrectUserNameOrPassword)?;
    auth_session.login(&user).await
      .map_err(|_| AuthError::Internal)?;
    Ok(())
  }.await.into()
}

async fn logout(mut auth_session: AuthSession) -> JsonResult<(), AuthError> {
  async move {
    auth_session.logout().await
      .map_err(|_| AuthError::Internal)?;
    Ok(())
  }.await.into()
}
