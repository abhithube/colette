use std::sync::Arc;

use chrono::{DateTime, Local, Utc};

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct CleanupJob(DateTime<Utc>);
impl From<DateTime<Utc>> for CleanupJob {
    fn from(value: DateTime<Utc>) -> Self {
        CleanupJob(value)
    }
}

pub struct CleanupService {
    cleanup_repository: Arc<dyn CleanupRepository>,
}

impl CleanupService {
    pub fn new(cleanup_repository: Arc<dyn CleanupRepository>) -> Self {
        Self { cleanup_repository }
    }

    pub async fn cleanup(&self) -> Result<(), Error> {
        let start = Local::now();
        println!("Started cleanup task at: {}", start);

        self.cleanup_repository.cleanup_feeds().await?;

        let elasped = (Local::now().time() - start.time()).num_milliseconds();
        println!("Finished cleanup task in {} ms", elasped);

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait CleanupRepository: Send + Sync {
    async fn cleanup_feeds(&self) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
