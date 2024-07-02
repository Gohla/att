use std::fmt;
use std::fmt::{Debug, Formatter};

use diesel::{Identifiable, insert_into, Insertable, OptionalExtension, Queryable, QueryDsl, Selectable};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::instrument;

use att_core::schema::users;

use crate::{DbError, DbPool};

#[derive(Clone)]
pub struct UsersDb {
  pool: DbPool
}

impl UsersDb {
  pub fn new(pool: DbPool) -> Self {
    Self { pool }
  }
}


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


impl UsersDb {
  #[instrument(skip(self), err)]
  pub async fn find(&self, user_id: i32) -> Result<Option<User>, DbError> {
    let user = self.pool.connect_and_query(move |conn| {
      users::table
        .find(user_id)
        .first(conn)
        .optional()
    }).await?;
    Ok(user)
  }

  #[instrument(skip(self), err)]
  pub async fn get_by_name(&self, user_name: String) -> Result<Option<User>, DbError> {
    let user = self.pool.connect_and_query(move |conn| {
      users::table
        .filter(users::name.eq(user_name))
        .first(conn)
        .optional()
    }).await?;
    Ok(user)
  }
}

#[derive(Insertable)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct NewUser {
  pub name: String,
  pub password_hash: String,
}

impl UsersDb {
  #[instrument(skip_all, fields(new_user.name = new_user.name), err)]
  pub async fn insert(&self, new_user: NewUser) -> Result<Option<User>, DbError> {
    let user = self.pool.connect_and_query(move |conn| {
      insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .optional()
    }).await?;
    Ok(user)
  }
}
