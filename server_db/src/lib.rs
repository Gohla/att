use std::marker::PhantomData;
use deadpool_diesel::postgres::{BuildError, InteractError, Manager, Object, Pool, PoolError, Runtime};
use diesel::PgConnection;
use thiserror::Error;

use att_core::run_or_compile_time_env;

pub mod users;
pub mod crates;

/// Database connection pool
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


/// Database connection, interaction, query error
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
  #[inline]
  pub async fn connect(&self) -> Result<DbPoolObj<M>, DbError> {
    let obj = self.pool.get().await?;
    Ok(DbPoolObj { obj, marker: self.marker })
  }

  #[inline]
  pub async fn interact<R: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.connect().await?.interact(f).await?;
    Ok(output)
  }

  #[inline]
  pub async fn perform<T: Send + 'static, E: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, E> + Send + 'static
  ) -> Result<T, DbError> where
    DbError: From<E>
  {
    let output = self.connect().await?.perform(f).await?;
    Ok(output)
  }

  #[inline]
  pub async fn query<T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, DbError> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.connect().await?.query(f).await?;
    Ok(output)
  }
}


/// Database connection pool object.
pub struct DbPoolObj<M> {
  obj: Object,
  marker: PhantomData<M>,
}

impl<M> DbPoolObj<M> {
  #[inline]
  pub async fn interact<R: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> R + Send + 'static
  ) -> Result<R, DbError> {
    let output = self.obj.interact(move |conn| f(&mut Self::db_conn(conn))).await?;
    Ok(output)
  }

  #[inline]
  pub async fn perform<T: Send + 'static, E: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, E> + Send + 'static
  ) -> Result<T, DbError> where
    DbError: From<E>
  {
    let output = self.obj.interact(move |conn| f(&mut Self::db_conn(conn))).await??;
    Ok(output)
  }

  #[inline]
  pub async fn query<T: Send + 'static>(
    &self,
    f: impl for<'c> FnOnce(&mut DbConn<'c, M>) -> Result<T, DbError> + Send + 'static
  ) -> Result<T, DbError> {
    let output = self.obj.interact(move |conn| f(&mut Self::db_conn(conn))).await??;
    Ok(output)
  }

  #[inline]
  fn db_conn(conn: &mut PgConnection) -> DbConn<M> { DbConn::new(conn) }
}


/// Database connection
pub struct DbConn<'c, M> {
  conn: &'c mut PgConnection,
  marker: PhantomData<M>,
}
impl<'c, M> DbConn<'c, M> {
  fn new(conn: &'c mut PgConnection) -> Self {
    Self { conn, marker: PhantomData }
  }
}


// pub trait DbPoolMethods {
//   type Obj: DbPoolObjMethods;
//   fn as_pool(&self) -> &DbPool;
//   fn convert_obj(obj: DbPoolObj) -> Self::Obj;
//
//   #[inline]
//   fn connect(&self) -> impl Future<Output=Result<Self::Obj, DbError>> {
//     async {
//       let obj = self.as_pool().pool.get().await?;
//       Ok(Self::convert_obj(DbPoolObj { obj }))
//     }
//   }
//
//   #[inline]
//   fn interact<R: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut <Self::Obj as DbPoolObjMethods>::Inner<'a>) -> R + Send + 'static
//   ) -> impl Future<Output=Result<R, DbError>> {
//     async {
//       let output = self.connect().await?.interact(f).await?;
//       Ok(output)
//     }
//   }
//
//   #[inline]
//   fn perform<T: Send + 'static, E: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut <Self::Obj as DbPoolObjMethods>::Inner<'a>) -> Result<T, E> + Send + 'static
//   ) -> impl Future<Output=Result<T, DbError>> where
//     DbError: From<E>
//   {
//     async {
//       let output = self.connect().await?.perform(f).await?;
//       Ok(output)
//     }
//   }
//
//   #[inline]
//   fn query<T: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut <Self::Obj as DbPoolObjMethods>::Inner<'a>) -> Result<T, DbError> + Send + 'static
//   ) -> impl Future<Output=Result<T, DbError>> {
//     async {
//       let output = self.connect().await?.query(f).await?;
//       Ok(output)
//     }
//   }
// }
//
// pub trait DbPoolObjMethods {
//   type Inner<'a>;
//   fn as_obj(&self) -> &DbPoolObj;
//   fn convert_conn<'a>(conn: &'a mut PgConnection) -> Self::Inner<'a>;
//
//   #[inline]
//   fn interact<R: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut Self::Inner<'a>) -> R + Send + 'static
//   ) -> impl Future<Output=Result<R, DbError>> {
//     async {
//       let output = self.as_obj().obj.interact(|conn| f(&mut Self::convert_conn(conn))).await?;
//       Ok(output)
//     }
//   }
//
//   #[inline]
//   fn perform<T: Send + 'static, E: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut Self::Inner<'a>) -> Result<T, E> + Send + 'static
//   ) -> impl Future<Output=Result<T, DbError>> where
//     DbError: From<E>
//   {
//     async {
//       let output = self.as_obj().obj.interact(|conn| f(&mut Self::convert_conn(conn))).await??;
//       Ok(output)
//     }
//   }
//
//   #[inline]
//   fn query<T: Send + 'static>(
//     &self,
//     f: impl for<'a> FnOnce(&mut Self::Inner<'a>) -> Result<T, DbError> + Send + 'static
//   ) -> impl Future<Output=Result<T, DbError>> {
//     async {
//       let output = self.as_obj().obj.interact(|conn| f(&mut Self::convert_conn(conn))).await??;
//       Ok(output)
//     }
//   }
// }

// impl DbPoolObj {
//   #[inline]
//   pub fn lock(&self) -> SyncGuard<'_, PgConnection> {
//     self.obj.lock().unwrap()
//   }
//
//   #[inline]
//   pub fn try_lock(&self) -> Option<SyncGuard<'_, PgConnection>> {
//     match self.obj.try_lock() {
//       Ok(l) => Some(l),
//       Err(TryLockError::WouldBlock) => None,
//       Err(TryLockError::Poisoned(e)) => panic!("{}", e),
//     }
//   }
// }
