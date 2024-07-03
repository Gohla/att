use std::fmt;
use std::fmt::{Debug, Formatter};

use diesel::{Identifiable, insert_into, Insertable, OptionalExtension, Queryable, QueryDsl, Selectable};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::instrument;

use att_core::schema::users;

use crate::{DbConn, DbError};

#[derive(Copy, Clone)]
pub struct UsersDb;

#[derive(Clone, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct User {
  pub id: i32,
  pub name: String,
  pub password_hash: String,
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


// Select users

impl DbConn<'_, UsersDb> {
  #[instrument(skip(self), err)]
  pub fn find(&mut self, user_id: i32) -> Result<Option<User>, DbError> {
    let user = users::table
      .find(user_id)
      .first(self.conn)
      .optional()?;
    Ok(user)
  }

  #[instrument(skip(self), err)]
  pub fn get_by_name(&mut self, user_name: &str) -> Result<Option<User>, DbError> {
    let user = users::table
      .filter(users::name.eq(user_name))
      .first(self.conn)
      .optional()?;
    Ok(user)
  }
}


// Insert users

#[derive(Insertable)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct NewUser {
  pub name: String,
  pub password_hash: String,
}

impl DbConn<'_, UsersDb> {
  #[instrument(skip_all, fields(new_user.name = new_user.name), err)]
  pub fn insert(&mut self, new_user: NewUser) -> Result<Option<User>, DbError> {
    let user = insert_into(users::table)
      .values(&new_user)
      .get_result(self.conn)
      .optional()?;
    Ok(user)
  }
}
