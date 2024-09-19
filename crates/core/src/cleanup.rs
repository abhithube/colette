use std::sync::Arc;

pub struct CleanupService {
    cleanup_repository: Arc<dyn CleanupRepository>,
}

impl CleanupService {
    pub fn new(cleanup_repository: Arc<dyn CleanupRepository>) -> Self {
        Self { cleanup_repository }
    }

    pub async fn cleanup_feeds(&self) -> Result<(), Error> {
        self.cleanup_repository.cleanup_feeds().await
    }

    pub async fn cleanup_tags(&self) -> Result<(), Error> {
        self.cleanup_repository.cleanup_tags().await
    }
}

#[async_trait::async_trait]
pub trait CleanupRepository: Send + Sync {
    async fn cleanup_feeds(&self) -> Result<(), Error>;

    async fn cleanup_tags(&self) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
