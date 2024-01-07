#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Debug, Formatter};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{self, SaltString};
use axum::{async_trait, Json, Router};
use axum_login::{AuthnBackend, AuthUser, UserId};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use att_core::users::UserCredentials;

use crate::data::Database;
use crate::util::F;

// Data

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct UsersData {
  next_id: u64,
  id_to_user: HashMap<u64, User>,
  name_to_id: HashMap<String, u64>,
}
impl Default for UsersData {
  fn default() -> Self {
    Self { next_id: 1, id_to_user: Default::default(), name_to_id: Default::default() }
  }
}
impl UsersData {
  fn contains_user_by_name(&self, user_name: &str) -> bool {
    self.name_to_id.contains_key(user_name)
  }

  fn get_user_by_id(&self, user_id: &u64) -> Option<&User> {
    self.id_to_user.get(user_id)
  }
  fn get_user_by_id_mut(&mut self, user_id: &u64) -> Option<&mut User> {
    self.id_to_user.get_mut(user_id)
  }
  fn get_user_by_name(&self, user_name: &str) -> Option<&User> {
    self.name_to_id.get(user_name).and_then(|id| self.get_user_by_id(id))
  }

  fn create_user(&mut self, name: String, password_hash: String) {
    let id = self.next_id;
    self.next_id += 1;
    self.create_user_with_id(id, name, password_hash);
  }
  fn create_user_with_id(&mut self, id: u64, name: String, password_hash: String) {
    let user = User::new(id, name.clone(), password_hash);
    self.id_to_user.insert(id, user);
    self.name_to_id.insert(name, id);
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  id: u64,
  name: String,
  password_hash: String,
}
impl User {
  fn new(id: u64, name: String, password_hash: String) -> Self {
    Self { id, name, password_hash }
  }
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

#[derive(Default, Clone)]
pub struct Users {
  argon2: Argon2<'static>,
}

impl Users {
  pub fn new(argon2: Argon2<'static>) -> Self {
    Self { argon2 }
  }

  pub fn ensure_default_user_exists(&self, data: &mut UsersData) -> Result<(), Box<dyn Error>> {
    let default_user_name = std::env::var("ATT_DEFAULT_USER_NAME")?;
    let default_password = std::env::var("ATT_DEFAULT_PASSWORD")?;
    let password_hash = self.hash_password(default_password.as_bytes())?;
    if let Some(user) = data.get_user_by_id_mut(&0) {
      user.name = default_user_name;
      user.password_hash = password_hash;
      // TODO: update `data.name_to_id` if the user name has changed
    } else {
      data.create_user_with_id(0, default_user_name, password_hash);
      // TODO: update `data.name_to_id` if the user name already exists
    }
    Ok(())
  }

  fn authenticate_user<'u>(
    &self,
    data: &'u UsersData,
    user_credentials: &UserCredentials
  ) -> Result<Option<&'u User>, password_hash::Error> {
    let user = if let Some(user) = data.get_user_by_name(&user_credentials.name) {
      let parsed_hash = PasswordHash::new(&user.password_hash)?;
      if self.argon2.verify_password(user_credentials.password.as_bytes(), &parsed_hash).is_ok() {
        Some(user)
      } else {
        None // TODO: properly handle error?
      }
    } else {
      None
    };
    Ok(user)
  }

  fn create_user(
    &self,
    data: &mut UsersData,
    user_credentials: UserCredentials
  ) -> Result<bool, password_hash::Error> {
    if data.contains_user_by_name(&user_credentials.name) {
      return Ok(false);
    }
    let password_hash = self.hash_password(user_credentials.password.as_bytes())?;
    data.create_user(user_credentials.name, password_hash);
    Ok(true)
  }

  fn hash_password(&self, password: &[u8]) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = self.argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
  }
}


// Authentication

#[derive(Clone)]
pub struct Authenticator {
  database: Database,
  users: Users,
}
impl Authenticator {
  pub fn new(database: Database, users: Users) -> Self { Self { database, users } }
}

impl AuthUser for User {
  type Id = u64;
  fn id(&self) -> Self::Id { self.id }
  fn session_auth_hash(&self) -> &[u8] { self.password_hash.as_bytes() }
}

#[async_trait]
impl AuthnBackend for Authenticator {
  type User = User;
  type Credentials = UserCredentials;
  type Error = password_hash::Error;
  async fn authenticate(&self, credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
    let users = &self.database.read().await.users;
    let user = self.users.authenticate_user(users, &credentials)?.cloned();
    Ok(user)
  }
  async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    let users = &self.database.read().await.users;
    let user = users.get_user_by_id(user_id).cloned();
    Ok(user)
  }
}

pub type AuthSession = axum_login::AuthSession<Authenticator>;


// Router

pub fn router() -> Router<()> {
  use axum::routing::post;
  Router::new()
    .route("/login", post(login).delete(logout))
}
async fn login(mut auth_session: AuthSession, Json(credentials): Json<UserCredentials>) -> Result<(), F> {
  let user = auth_session.authenticate(credentials.clone()).await?.ok_or(F::forbidden())?;
  auth_session.login(&user).await?;
  Ok(())
}
async fn logout(mut auth_session: AuthSession) -> Result<(), F> {
  auth_session.logout().await?;
  Ok(())
}
