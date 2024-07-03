use std::marker::PhantomData;

use deadpool_diesel::postgres::{BuildError, InteractError, Manager, Object, Pool, PoolError, Runtime};
use diesel::PgConnection;
use thiserror::Error;

use att_core::run_or_compile_time_env;

pub mod users;
pub mod crates;

/// Database connection pool.
#[derive(Clone)]
pub struct DbPool<M = ()> {
  pool: Pool,
  marker: PhantomData<M>,
}
impl DbPool {
  pub fn new() -> Result<Self, BuildError> {
    let manager = Manager::new(run_or_compile_time_env!("DATABASE_URL"), Runtime::Tokio1);
    let pool = Pool::builder(manager)
      .max_size(8)
      .build()?;
    let db = Self { pool, marker: PhantomData };
    Ok(db)
  }

  #[inline]
  pub fn with<MM>(&self) -> DbPool<MM> {
    DbPool { pool: self.pool.clone(), marker: PhantomData }
  }
}


/// Database connection, interaction, or query error.
#[derive(Debug, Error)]
pub enum DbError {
  #[error("Database query failed: {0}")]
  Query(#[from] diesel::result::Error),
  #[error("Failed to get database connection from pool: {0}")]
  ConnectionFromPool(#[from] PoolError),
  #[error("Performing operation with database connection panicked: {0}")]
  PerformPanic(String),
  #[error("Performing operation with database connection panicked, but the panic does not contain a message")]
  PerformPanicNoMessage,
  #[error("Performing operation with database connection was aborted")]
  PerformAbort,
}
impl From<InteractError> for DbError {
  fn from(error: InteractError) -> Self {
    match error {
      InteractError::Panic(e) => {
        if let Ok(message) = e.downcast::<String>() {
          DbError::PerformPanic(*message)
        } else {
          DbError::PerformPanicNoMessage
        }
      },
      InteractError::Aborted => DbError::PerformAbort,
    }
  }
}

impl<M> DbPool<M> {
  /// Obtain a database connection pool object from the pool.
  #[inline]
  pub async fn get(&self) -> Result<DbPoolObj<M>, DbError> {
    let obj = self.pool.get().await?;
    Ok(DbPoolObj { obj, marker: self.marker })
  }

  /// Interact synchronously with `f` that returns `R`, on a database connection newly obtained from the pool.
  #[inline]
  pub async fn interact<R: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.get().await?.interact(f).await?;
    Ok(output)
  }

  /// Query synchronously with `f` that returns `Result<T, DbError>`, on a database connection newly obtained from the
  /// pool.
  #[inline]
  pub async fn query<T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, DbError> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.get().await?.query(f).await?;
    Ok(output)
  }

  /// Perform `f` synchronously with `f` returning `Result<T, E>` where `E: From<DbError>`, on a database connection
  /// newly obtained from the pool.
  #[inline]
  pub async fn perform<E: Send + 'static, T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, E> + Send + 'static
  ) -> Result<T, E> where
    E: From<DbError>
  {
    let output = self.get().await?.perform(f).await?;
    Ok(output)
  }
}


/// Database connection pool object.
pub struct DbPoolObj<M> {
  obj: Object,
  marker: PhantomData<M>,
}

impl<M> DbPoolObj<M> {
  /// Interact synchronously with `f` that returns `R`.
  #[inline]
  pub async fn interact<R: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.obj.interact(move |conn| f(&mut DbConn::new(conn))).await?;
    Ok(output)
  }

  /// Query synchronously with `f` that returns `Result<T, DbError>`.
  #[inline]
  pub async fn query<T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, DbError> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.interact(f).await??;
    Ok(output)
  }

  /// Perform `f` synchronously, with `f` returning `Result<T, E>` where `E: From<DbError>`.
  #[inline]
  pub async fn perform<E: Send + 'static, T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, E> + Send + 'static
  ) -> Result<T, E> where
    E: From<DbError>
  {
    let output = self.interact(f).await??;
    Ok(output)
  }
}


/// Database connection
pub struct DbConn<'c, M> {
  conn: &'c mut PgConnection,
  marker: PhantomData<M>,
}
impl<'c, M> DbConn<'c, M> {
  #[inline]
  fn new(conn: &'c mut PgConnection) -> Self { Self { conn, marker: PhantomData } }

  #[inline]
  pub fn inner(&'c mut self) -> &'c mut PgConnection { &mut self.conn }
}
