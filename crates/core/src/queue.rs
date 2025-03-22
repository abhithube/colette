use uuid::Uuid;

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
