#![allow(dead_code)]

use std::io;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use att_core::app::storage::{DirectoryKind, Storage};

use crate::crates::CratesData;
use crate::job_scheduler::{BlockingJob, JobAction, JobResult};
use crate::users::UsersData;

#[derive(Default, Clone, Debug)]
pub struct Database(Arc<RwLock<Data>>);

impl Database {
  pub async fn read(&self) -> RwLockReadGuard<Data> {
    self.0.read().await
  }
  pub fn blocking_read(&self) -> RwLockReadGuard<Data> {
    self.0.blocking_read()
  }
  pub async fn write(&self) -> RwLockWriteGuard<Data> {
    self.0.write().await
  }
  pub fn blocking_write(&self) -> RwLockWriteGuard<Data> {
    self.0.blocking_write()
  }

  pub fn blocking_deserialize(start: &Storage) -> Result<Self, io::Error> {
    let data = Data::deserialize(start)?;
    Ok(Self(Arc::new(RwLock::new(data))))
  }
  pub async fn serialize(&self, start: &Storage) -> Result<(), io::Error> {
    self.read().await.serialize(start)?;
    Ok(())
  }
  pub fn blocking_serialize(&self, start: &Storage) -> Result<(), io::Error> {
    self.blocking_read().serialize(start)?;
    Ok(())
  }
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  pub users: UsersData,
  pub crates: CratesData,
}

impl Data {
  fn deserialize(start: &Storage) -> Result<Self, io::Error> {
    Ok(start.deserialize_json_file(DirectoryKind::Data, "data.json")?.unwrap_or_default())
  }
  fn serialize(&self, start: &Storage) -> Result<(), io::Error> {
    start.serialize_json_file(DirectoryKind::Data, "data.json", self)?;
    Ok(())
  }
}

pub struct StoreDatabaseJob {
  start: Storage,
  database: Database,
}
impl StoreDatabaseJob {
  pub fn new(start: Storage, database: Database) -> Self {
    Self { start, database }
  }
}
impl BlockingJob for StoreDatabaseJob {
  fn run(&self) -> JobResult {
    self.database.blocking_serialize(&self.start)?;
    Ok(JobAction::Continue)
  }
}
