use std::time::Duration;

use apalis_core::{
    request::{Parts, Request},
    task::task_id::TaskId,
};
use colette_core::job::{Error, Storage, StorageContext};
pub use redis::RedisStorage;
use serde::{de::DeserializeOwned, Serialize};
pub use sqlite::SqliteStorage;

mod redis;
mod sqlite;

#[derive(Clone)]
pub enum JobAdapter<T> {
    Sqlite(SqliteStorage<T>),
    Redis(RedisStorage<T>),
}

#[async_trait::async_trait]
impl<T: Send + Sync + Unpin + 'static + Serialize + DeserializeOwned> Storage<T> for JobAdapter<T> {
    async fn push_request(
        &mut self,
        req: Request<T, StorageContext>,
    ) -> Result<Parts<StorageContext>, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.push_request(req).await,
            Self::Redis(ref mut storage) => storage.push_request(req).await,
        }
    }

    async fn schedule_request(
        &mut self,
        request: Request<T, StorageContext>,
        on: i64,
    ) -> Result<Parts<StorageContext>, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.schedule_request(request, on).await,
            Self::Redis(ref mut storage) => storage.schedule_request(request, on).await,
        }
    }

    async fn len(&mut self) -> Result<i64, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.len().await,
            Self::Redis(ref mut storage) => storage.len().await,
        }
    }

    async fn fetch_by_id(
        &mut self,
        job_id: &TaskId,
    ) -> Result<Option<Request<T, StorageContext>>, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.fetch_by_id(job_id).await,
            Self::Redis(ref mut storage) => storage.fetch_by_id(job_id).await,
        }
    }

    async fn update(&mut self, job: Request<T, StorageContext>) -> Result<(), Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.update(job).await,
            Self::Redis(ref mut storage) => storage.update(job).await,
        }
    }

    async fn reschedule(
        &mut self,
        job: Request<T, StorageContext>,
        wait: Duration,
    ) -> Result<(), Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.reschedule(job, wait).await,
            Self::Redis(ref mut storage) => storage.reschedule(job, wait).await,
        }
    }

    async fn is_empty(&mut self) -> Result<bool, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.is_empty().await,
            Self::Redis(ref mut storage) => storage.is_empty().await,
        }
    }

    async fn vacuum(&mut self) -> Result<usize, Error> {
        match self {
            Self::Sqlite(ref mut storage) => storage.vacuum().await,
            Self::Redis(ref mut storage) => storage.vacuum().await,
        }
    }
}
