use std::time::Duration;

use apalis_core::{
    backend::Backend,
    layers::{AckLayer, Service},
    poller::{stream::BackendStream, Poller},
    request::{Parts, Request, RequestStream},
    response::Response,
    storage::Storage as _,
    task::task_id::TaskId,
    worker::{Context, Worker},
};
use apalis_redis::{Config, RedisContext, RedisStorage as ApalisStorage};
use colette_core::job::{Error, Storage, StorageContext};
use futures::channel::mpsc::Sender;
use redis::aio::MultiplexedConnection;
use serde::{de::DeserializeOwned, Serialize};

pub struct RedisStorage<T> {
    inner: ApalisStorage<T, MultiplexedConnection>,
}

impl<T> Clone for RedisStorage<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> RedisStorage<T> {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self {
            inner: ApalisStorage::new(conn),
        }
    }

    pub fn new_with_config(conn: MultiplexedConnection, config: Config) -> Self {
        Self {
            inner: ApalisStorage::new_with_config(conn, config),
        }
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync + Unpin + 'static + Serialize + DeserializeOwned> Storage<T>
    for RedisStorage<T>
{
    async fn push_request(
        &mut self,
        request: Request<T, StorageContext>,
    ) -> Result<Parts<StorageContext>, Error> {
        let _parts = self
            .inner
            .push_request(Request::new_with_ctx(request.args, RedisContext::default()))
            .await?;

        let mut parts = Parts::default();
        parts.context = StorageContext::default();

        Ok(parts)
    }

    async fn schedule_request(
        &mut self,
        request: Request<T, StorageContext>,
        on: i64,
    ) -> Result<Parts<StorageContext>, Error> {
        let _parts = self
            .inner
            .schedule_request(
                Request::new_with_ctx(request.args, RedisContext::default()),
                on,
            )
            .await?;

        let mut parts = Parts::default();
        parts.context = StorageContext::default();

        Ok(parts)
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

        Ok(request.map(|e| Request::new_with_ctx(e.args, StorageContext::default())))
    }

    async fn update(&mut self, request: Request<T, StorageContext>) -> Result<(), Error> {
        self.inner
            .update(Request::new_with_ctx(request.args, RedisContext::default()))
            .await?;

        Ok(())
    }

    async fn reschedule(
        &mut self,
        request: Request<T, StorageContext>,
        wait: Duration,
    ) -> Result<(), Error> {
        self.inner
            .reschedule(
                Request::new_with_ctx(request.args, RedisContext::default()),
                wait,
            )
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

impl<
        T: Send + Sync + Unpin + 'static + Serialize + DeserializeOwned,
        Res: Send + Sync + 'static + Serialize,
    > Backend<Request<T, RedisContext>, Res> for RedisStorage<T>
{
    type Stream = BackendStream<RequestStream<Request<T, RedisContext>>>;
    type Layer = AckLayer<Sender<(RedisContext, Response<Res>)>, T, RedisContext, Res>;

    fn poll<Svc: Service<Request<T, RedisContext>, Response = Res>>(
        self,
        worker: &Worker<Context>,
    ) -> Poller<Self::Stream, Self::Layer> {
        self.inner.poll::<Svc>(worker)
    }
}
