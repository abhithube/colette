use std::{any::type_name, time::Duration};

use apalis_core::{
    backend::Backend,
    codec::{json::JsonCodec, Codec},
    layers::{AckLayer, Service},
    poller::{stream::BackendStream, Poller},
    request::{Parts, Request, RequestStream},
    storage::Storage as _,
    task::task_id::TaskId,
    worker::{Context, Worker},
};
use apalis_sql::{context::SqlContext, sqlite::SqliteStorage, Config};
use chrono::DateTime;
use colette_core::worker::{Error, Storage, StorageContext};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{Pool, Sqlite};

pub struct SqliteStorageAdapter<T, C = JsonCodec<String>> {
    inner: SqliteStorage<T, C>,
}

impl<T> Clone for SqliteStorageAdapter<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> SqliteStorageAdapter<T> {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            inner: SqliteStorage::new_with_config(
                pool,
                Config::new(type_name::<T>()).set_poll_interval(Duration::from_secs(1)),
            ),
        }
    }

    pub fn new_with_config(pool: Pool<Sqlite>, config: Config) -> Self {
        Self {
            inner: SqliteStorage::new_with_config(pool, config),
        }
    }
}

impl SqliteStorageAdapter<()> {
    pub async fn setup(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        SqliteStorage::setup(pool).await
    }
}

#[async_trait::async_trait]
impl<
        T: Send + Sync + Unpin + 'static + Serialize + DeserializeOwned,
        C: Send + Codec<Compact = String>,
    > Storage<T> for SqliteStorageAdapter<T, C>
{
    async fn push_request(
        &mut self,
        request: Request<T, StorageContext>,
    ) -> Result<Parts<StorageContext>, Error> {
        let parts = self.inner.push_request(SqlRequest::from(request).0).await?;

        Ok(Parts::from(SqlParts(parts)))
    }

    async fn schedule_request(
        &mut self,
        request: Request<T, StorageContext>,
        on: i64,
    ) -> Result<Parts<StorageContext>, Error> {
        let parts = self
            .inner
            .schedule_request(SqlRequest::from(request).0, on)
            .await?;

        Ok(Parts::from(SqlParts(parts)))
    }

    async fn len(&mut self) -> Result<i64, Error> {
        let len = self.inner.len().await?;

        Ok(len)
    }

    async fn fetch_by_id(
        &mut self,
        job_id: &TaskId,
    ) -> Result<Option<Request<T, StorageContext>>, Error> {
        let request = self.inner.fetch_by_id(job_id).await?;

        Ok(request.map(|e| Request::new_with_parts(e.args, Parts::from(SqlParts(e.parts)))))
    }

    async fn update(&mut self, request: Request<T, StorageContext>) -> Result<(), Error> {
        self.inner.update(SqlRequest::from(request).0).await?;

        Ok(())
    }

    async fn reschedule(
        &mut self,
        request: Request<T, StorageContext>,
        wait: Duration,
    ) -> Result<(), Error> {
        self.inner
            .reschedule(SqlRequest::from(request).0, wait)
            .await?;

        Ok(())
    }

    async fn is_empty(&mut self) -> Result<bool, Error> {
        let is_empty = self.inner.is_empty().await?;

        Ok(is_empty)
    }

    async fn vacuum(&mut self) -> Result<usize, Error> {
        let count = self.inner.vacuum().await?;

        Ok(count)
    }
}

impl<T: Send + Sync + Unpin + 'static + Serialize + DeserializeOwned, Res>
    Backend<Request<T, SqlContext>, Res> for SqliteStorageAdapter<T>
{
    type Stream = BackendStream<RequestStream<Request<T, SqlContext>>>;
    type Layer = AckLayer<SqliteStorage<T>, T, SqlContext, Res>;

    fn poll<Svc: Service<Request<T, SqlContext>, Response = Res>>(
        self,
        worker: &Worker<Context>,
    ) -> Poller<Self::Stream, Self::Layer> {
        self.inner.poll::<Svc>(worker)
    }
}

struct SqlParts(Parts<SqlContext>);
struct SqlRequest<T>(Request<T, SqlContext>);

impl<T> From<Request<T, StorageContext>> for SqlRequest<T> {
    fn from(value: Request<T, StorageContext>) -> Self {
        let (args, old_parts) = value.take_parts();
        let old_ctx = old_parts.context;

        let mut ctx = SqlContext::new();
        ctx.set_status(old_ctx.status);
        ctx.set_run_at(old_ctx.start_at);
        ctx.set_max_attempts(old_ctx.max_attempts);
        ctx.set_last_error(old_ctx.last_error);
        ctx.set_lock_at(old_ctx.locked_at.map(|e| e.timestamp()));
        ctx.set_lock_by(old_ctx.locked_by);
        ctx.set_done_at(old_ctx.completed_at.map(|e| e.timestamp()));

        let mut parts = Parts::default();
        parts.task_id = old_parts.task_id;
        parts.data = old_parts.data;
        parts.attempt = old_parts.attempt;
        parts.context = ctx;
        parts.namespace = old_parts.namespace;

        Self(Request::new_with_parts(args, parts))
    }
}

impl From<SqlParts> for Parts<StorageContext> {
    fn from(value: SqlParts) -> Self {
        let old_parts = value.0;
        let old_ctx = old_parts.context;

        let ctx = StorageContext {
            status: old_ctx.status().to_owned(),
            start_at: *old_ctx.run_at(),
            max_attempts: old_ctx.max_attempts(),
            last_error: old_ctx.last_error().to_owned(),
            locked_at: old_ctx
                .lock_at()
                .and_then(|e| DateTime::from_timestamp(e, 0))
                .to_owned(),
            locked_by: old_ctx.lock_by().to_owned(),
            completed_at: old_ctx
                .done_at()
                .and_then(|e| DateTime::from_timestamp(e, 0))
                .to_owned(),
        };

        let mut parts = Parts::default();
        parts.task_id = old_parts.task_id;
        parts.data = old_parts.data;
        parts.attempt = old_parts.attempt;
        parts.context = ctx;
        parts.namespace = old_parts.namespace;

        parts
    }
}
