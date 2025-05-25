#[cfg(not(any(feature = "local")))]
compile_error!("at least one of 'local' must be enabled");

#[cfg(feature = "local")]
pub use local::LocalQueue;
use uuid::Uuid;

#[cfg(feature = "local")]
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
    #[cfg(feature = "local")]
    Local(local::LocalJobProducer),
}

#[async_trait::async_trait]
impl JobProducer for JobProducerAdapter {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error> {
        match self {
            #[cfg(feature = "local")]
            Self::Local(producer) => producer.push(job_id).await,
        }
    }
}

#[derive(Debug)]
pub enum JobConsumerAdapter {
    #[cfg(feature = "local")]
    Local(local::LocalJobConsumer),
}

#[async_trait::async_trait]
impl JobConsumer for JobConsumerAdapter {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error> {
        match self {
            #[cfg(feature = "local")]
            Self::Local(consumer) => consumer.pop().await,
        }
    }
}
