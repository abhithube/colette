use std::sync::Arc;

pub struct CleanupInfo {
    pub feed_count: u64,
    pub feed_entry_count: u64,
}

pub struct CleanupService {
    cleanup_repository: Arc<dyn CleanupRepository>,
}

impl CleanupService {
    pub fn new(cleanup_repository: Arc<dyn CleanupRepository>) -> Self {
        Self { cleanup_repository }
    }

    pub async fn cleanup(&self) -> Result<CleanupInfo, Error> {
        let info = self.cleanup_repository.cleanup_feeds().await?;

        Ok(CleanupInfo {
            feed_count: info.feed_count,
            feed_entry_count: info.feed_entry_count,
        })
    }
}

#[async_trait::async_trait]
pub trait CleanupRepository: Send + Sync {
    async fn cleanup_feeds(&self) -> Result<FeedCleanupInfo, Error>;
}

pub struct FeedCleanupInfo {
    pub feed_count: u64,
    pub feed_entry_count: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
