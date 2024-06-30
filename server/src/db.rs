use deadpool_diesel::InteractError;
use deadpool_diesel::postgres::{Pool, PoolError};
use thiserror::Error;

pub type DbPool = Pool;
pub type DbPoolError = PoolError;
pub type DbInteractError = InteractError;

#[derive(Debug, Error)]
pub enum DbError {
  #[error("Failed to create database connection from pool")]
  DbConnection(#[from] DbPoolError),
  #[error("Failed to performing operation with database connection")]
  DbInteract,
  #[error("Query failed")]
  Query(#[from] diesel::result::Error),
}

impl From<DbInteractError> for DbError {
  fn from(_value: DbInteractError) -> Self {
    // TODO: this discards the panic
    Self::DbInteract
  }
}
