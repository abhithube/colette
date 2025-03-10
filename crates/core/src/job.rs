use std::time::Duration;

use apalis_core::{
    backend::{Stat, WorkerState},
    request::{Parts, Request, State},
    service_fn::FromRequest,
    task::task_id::TaskId,
    worker::{Worker, WorkerId},
};
use chrono::{DateTime, Utc};
use redis::RedisError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageContext {
    pub start_at: DateTime<Utc>,
    pub max_attempts: i32,
    pub status: State,
    pub locked_by: Option<WorkerId>,
    pub locked_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl Default for StorageContext {
    fn default() -> Self {
        Self {
            start_at: Utc::now(),
            max_attempts: 25,
            status: State::Pending,
            locked_by: None,
            locked_at: None,
            completed_at: None,
            last_error: None,
        }
    }
}

impl<Req> FromRequest<Request<Req, StorageContext>> for StorageContext {
    fn from_request(req: &Request<Req, StorageContext>) -> Result<Self, apalis_core::error::Error> {
        Ok(req.parts.context.clone())
    }
}

/// Represents a [Storage] that can persist a request.
#[async_trait::async_trait]
pub trait Storage<T: Send + 'static>: Send {
    /// Pushes a job to a storage
    async fn push(&mut self, job: T) -> Result<Parts<StorageContext>, Error> {
        self.push_request(Request::new(job)).await
    }

    /// Pushes a constructed request to a storage
    async fn push_request(
        &mut self,
        req: Request<T, StorageContext>,
    ) -> Result<Parts<StorageContext>, Error>;

    /// Push a job with defaults into the scheduled set
    async fn schedule(&mut self, job: T, on: i64) -> Result<Parts<StorageContext>, Error> {
        self.schedule_request(Request::new(job), on).await
    }

    /// Push a request into the scheduled set
    async fn schedule_request(
        &mut self,
        request: Request<T, StorageContext>,
        on: i64,
    ) -> Result<Parts<StorageContext>, Error>;

    /// Return the number of pending jobs from the queue
    async fn len(&mut self) -> Result<i64, Error>;

    /// Fetch a job given an id
    async fn fetch_by_id(
        &mut self,
        job_id: &TaskId,
    ) -> Result<Option<Request<T, StorageContext>>, Error>;

    /// Update a job details
    async fn update(&mut self, job: Request<T, StorageContext>) -> Result<(), Error>;

    /// Reschedule a job
    async fn reschedule(
        &mut self,
        job: Request<T, StorageContext>,
        wait: Duration,
    ) -> Result<(), Error>;

    /// Returns true if there is no jobs in the storage
    async fn is_empty(&mut self) -> Result<bool, Error>;

    /// Vacuum the storage, removes done and killed jobs
    async fn vacuum(&mut self) -> Result<usize, Error>;
}

/// Represents functionality that allows reading of jobs and stats from a backend
/// Some backends esp MessageQueues may not currently implement this
#[async_trait::async_trait]
pub trait BackendExpose<T>
where
    Self: Sized,
{
    /// The request type being handled by the backend
    type Request;

    /// List all Workers that are working on a backend
    async fn list_workers(&self) -> Result<Vec<Worker<WorkerState>>, Error>;

    /// Returns the counts of jobs in different states
    async fn stats(&self) -> Result<Stat, Error>;

    /// Fetch jobs persisted in a backend
    async fn list_jobs(&self, status: &State, page: i32) -> Result<Vec<Self::Request>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Redis(#[from] RedisError),
}
