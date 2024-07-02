use deadpool_diesel::postgres::{BuildError, Connection, InteractError, Manager, Pool, PoolError, Runtime};
use diesel::PgConnection;
use thiserror::Error;

use att_core::run_or_compile_time_env;

pub mod users;
pub mod crates;

#[derive(Clone)]
pub struct DbPool {
  pool: Pool,
}

impl DbPool {
  pub fn new() -> Result<Self, BuildError> {
    let manager = Manager::new(run_or_compile_time_env!("DATABASE_URL"), Runtime::Tokio1);
    let pool = Pool::builder(manager)
      .max_size(8)
      .build()?;
    let db = Self { pool };
    Ok(db)
  }
}

pub struct DbConn {
  connection: Connection,
}

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

impl DbPool {
  #[inline]
  pub async fn connect(&self) -> Result<DbConn, DbError> {
    let connection = self.pool.get().await?;
    Ok(DbConn { connection })
  }

  #[inline]
  pub async fn connect_and_interact<R: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.connect().await?.interact(f).await?;
    Ok(output)
  }

  #[inline]
  pub async fn connect_and_perform<T: Send + 'static, E: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> Result<T, E> + Send + 'static
  ) -> Result<T, DbError> where
    DbError: From<E>
  {
    let output = self.connect().await?.perform(f).await?;
    Ok(output)
  }

  #[inline]
  pub async fn connect_and_query<T: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.connect().await?.perform(f).await?;
    Ok(output)
  }
}

impl DbConn {
  #[inline]
  pub async fn interact<R: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.connection.interact(f).await?;
    Ok(output)
  }

  #[inline]
  pub async fn perform<T: Send + 'static, E: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> Result<T, E> + Send + 'static
  ) -> Result<T, DbError> where
    DbError: From<E>
  {
    let output = self.connection.interact(f).await??;
    Ok(output)
  }

  #[inline]
  pub async fn query<T: Send + 'static>(
    &self,
    f: impl FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.connection.interact(f).await??;
    Ok(output)
  }
}
