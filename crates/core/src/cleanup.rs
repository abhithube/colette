use std::sync::Arc;

use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct CleanupJob(DateTime<Utc>);
impl From<DateTime<Utc>> for CleanupJob {
    fn from(value: DateTime<Utc>) -> Self {
        CleanupJob(value)
    }
}

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
