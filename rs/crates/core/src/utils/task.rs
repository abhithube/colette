use async_trait::async_trait;

#[async_trait]
pub trait Task: Send + Sync {
    async fn run(&self) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
