use colette_core::queue::{Error, JobConsumer, JobProducer};
pub use local::LocalQueue;
use local::{LocalJobConsumer, LocalJobProducer};
use uuid::Uuid;

mod local;

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
