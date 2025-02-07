use std::time::Duration;

use apalis_core::{
    request::{Parts, Request},
    storage::Storage as ApalisStorage,
    task::task_id::TaskId,
};
use apalis_redis::{RedisContext, RedisError};

/// Represents a [Storage] that can persist a request.
#[async_trait::async_trait]
pub trait Storage: Send {
    /// The type of job that can be persisted
    type Job: Send;

    /// The error produced by the storage
    type Error;

    /// This is the type that storages store as the metadata related to a job
    type Context: Default;

    /// Pushes a job to a storage
    async fn push(&mut self, job: Self::Job) -> Result<Parts<Self::Context>, Self::Error> {
        self.push_request(Request::new(job)).await
    }

    /// Pushes a constructed request to a storage
    async fn push_request(
        &mut self,
        req: Request<Self::Job, Self::Context>,
    ) -> Result<Parts<Self::Context>, Self::Error>;

    /// Push a job with defaults into the scheduled set
    async fn schedule(
        &mut self,
        job: Self::Job,
        on: i64,
    ) -> Result<Parts<Self::Context>, Self::Error> {
        self.schedule_request(Request::new(job), on).await
    }

    /// Push a request into the scheduled set
    async fn schedule_request(
        &mut self,
        request: Request<Self::Job, Self::Context>,
        on: i64,
    ) -> Result<Parts<Self::Context>, Self::Error>;

    /// Return the number of pending jobs from the queue
    async fn len(&mut self) -> Result<i64, Self::Error>;

    /// Fetch a job given an id
    async fn fetch_by_id(
        &mut self,
        job_id: &TaskId,
    ) -> Result<Option<Request<Self::Job, Self::Context>>, Self::Error>;

    /// Update a job details
    async fn update(&mut self, job: Request<Self::Job, Self::Context>) -> Result<(), Self::Error>;

    /// Reschedule a job
    async fn reschedule(
        &mut self,
        job: Request<Self::Job, Self::Context>,
        wait: Duration,
    ) -> Result<(), Self::Error>;

    /// Returns true if there is no jobs in the storage
    async fn is_empty(&mut self) -> Result<bool, Self::Error>;

    /// Vacuum the storage, removes done and killed jobs
    async fn vacuum(&mut self) -> Result<usize, Self::Error>;
}

pub type DynStorage<T> = dyn Storage<Job = T, Context = RedisContext, Error = RedisError>;

#[async_trait::async_trait]
impl<T: ApalisStorage + Send> Storage for T
where
    T::Job: Send,
    T::Context: Send,
{
    type Job = T::Job;
    type Error = T::Error;
    type Context = T::Context;

    async fn push_request(
        &mut self,
        req: Request<Self::Job, Self::Context>,
    ) -> Result<Parts<Self::Context>, Self::Error> {
        self.push_request(req).await
    }

    async fn schedule_request(
        &mut self,
        request: Request<Self::Job, Self::Context>,
        on: i64,
    ) -> Result<Parts<Self::Context>, Self::Error> {
        self.schedule_request(request, on).await
    }

    async fn len(&mut self) -> Result<i64, Self::Error> {
        self.len().await
    }

    async fn fetch_by_id(
        &mut self,
        job_id: &TaskId,
    ) -> Result<Option<Request<Self::Job, Self::Context>>, Self::Error> {
        self.fetch_by_id(job_id).await
    }

    async fn update(&mut self, job: Request<Self::Job, Self::Context>) -> Result<(), Self::Error> {
        self.update(job).await
    }

    async fn reschedule(
        &mut self,
        job: Request<Self::Job, Self::Context>,
        wait: Duration,
    ) -> Result<(), Self::Error> {
        self.reschedule(job, wait).await
    }

    async fn is_empty(&mut self) -> Result<bool, Self::Error> {
        self.is_empty().await
    }

    async fn vacuum(&mut self) -> Result<usize, Self::Error> {
        self.vacuum().await
    }
}
