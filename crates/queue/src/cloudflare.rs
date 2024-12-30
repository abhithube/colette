use std::marker::PhantomData;

use serde::Serialize;
use worker::Queue as CfQueue;

use crate::Queue;

#[derive(Clone)]
pub struct CloudflareQueue<T> {
    inner: CfQueue,
    _data: PhantomData<T>,
}

impl<T> CloudflareQueue<T> {
    pub fn new(inner: CfQueue) -> Self {
        Self {
            inner,
            _data: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T: Clone + Send + Sync + Serialize + 'static> Queue for CloudflareQueue<T> {
    type Data = T;

    async fn push(&self, data: Self::Data) -> Result<(), anyhow::Error> {
        send(data, &self.inner).await.map_err(|e| e.into())
    }
}

#[worker::send]
async fn send<T: Serialize>(data: T, queue: &CfQueue) -> worker::Result<()> {
    queue.send(data).await
}
