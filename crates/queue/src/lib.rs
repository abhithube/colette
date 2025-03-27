pub use local::LocalQueue;
use local::{LocalJobConsumer, LocalJobProducer};
use uuid::Uuid;

mod local;

#[async_trait::async_trait]
pub trait JobProducer: Send + Sync + 'static {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait JobConsumer: Send + Sync + 'static {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("backend error: {0}")]
    Backend(String),
}

#[derive(Debug, Clone)]
pub enum JobProducerAdapter {
    Local(LocalJobProducer),
}

#[async_trait::async_trait]
impl JobProducer for JobProducerAdapter {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error> {
        match self {
            Self::Local(producer) => producer.push(job_id).await,
        }
    }
}

#[derive(Debug)]
pub enum JobConsumerAdapter {
    Local(LocalJobConsumer),
}

#[async_trait::async_trait]
impl JobConsumer for JobConsumerAdapter {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error> {
        match self {
            Self::Local(consumer) => consumer.pop().await,
        }
    }
}
