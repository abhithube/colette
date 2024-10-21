use std::time::Duration;

use apalis::prelude::{
    BackendStream, Job, Poller, Request, RequestStream, Storage, TaskId, WorkerId,
};
use apalis_core::{
    layers::{Ack, AckLayer},
    Backend,
};
#[cfg(feature = "postgres")]
use apalis_sql::postgres::PostgresStorage;
#[cfg(feature = "sqlite")]
use apalis_sql::sqlite::SqliteStorage;
pub use cleanup::cleanup;
pub use refresh::refresh_feeds;
use serde::{de::DeserializeOwned, Serialize};

mod cleanup;
mod refresh;

#[derive(Clone)]
pub enum WorkerStorage<T: Clone> {
    #[cfg(feature = "postgres")]
    Postgres(PostgresStorage<T>),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStorage<T>),
}

impl<T: Clone + Send + Sync + Unpin + Serialize + DeserializeOwned + Job + 'static> Storage
    for WorkerStorage<T>
{
    type Job = T;

    type Error = sqlx::Error;

    type Identifier = TaskId;

    async fn push(&mut self, job: Self::Job) -> Result<Self::Identifier, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.push(job).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.push(job).await,
        }
    }

    async fn schedule(&mut self, job: Self::Job, on: i64) -> Result<Self::Identifier, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.schedule(job, on).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.schedule(job, on).await,
        }
    }

    async fn len(&self) -> Result<i64, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.len().await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.len().await,
        }
    }

    async fn fetch_by_id(
        &self,
        job_id: &Self::Identifier,
    ) -> Result<Option<Request<Self::Job>>, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.fetch_by_id(job_id).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.fetch_by_id(job_id).await,
        }
    }

    async fn update(&self, job: Request<Self::Job>) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.update(job).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.update(job).await,
        }
    }

    async fn reschedule(
        &mut self,
        job: Request<Self::Job>,
        wait: Duration,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.reschedule(job, wait).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.reschedule(job, wait).await,
        }
    }

    async fn is_empty(&self) -> Result<bool, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.is_empty().await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.is_empty().await,
        }
    }

    async fn vacuum(&self) -> Result<usize, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.vacuum().await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.vacuum().await,
        }
    }
}

impl<T: Clone + Send + Sync + Unpin + Serialize + DeserializeOwned + Job + 'static>
    Backend<Request<T>> for WorkerStorage<T>
{
    type Stream = BackendStream<RequestStream<Request<T>>>;

    type Layer = AckLayer<WorkerStorage<T>, T>;

    fn common_layer(&self, worker: WorkerId) -> Self::Layer {
        AckLayer::new((*self).clone(), worker)
    }

    fn poll(self, worker: WorkerId) -> Poller<Self::Stream> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.poll(worker),
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.poll(worker),
        }
    }
}

impl<T: Clone + Send + Sync> Ack<T> for WorkerStorage<T> {
    type Acknowledger = TaskId;

    type Error = sqlx::Error;

    async fn ack(
        &self,
        worker_id: &WorkerId,
        data: &Self::Acknowledger,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WorkerStorage::Postgres(storage) => storage.ack(worker_id, data).await,
            #[cfg(feature = "sqlite")]
            WorkerStorage::Sqlite(storage) => storage.ack(worker_id, data).await,
        }
    }
}
