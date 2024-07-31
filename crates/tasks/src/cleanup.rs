use std::sync::Arc;

use colette_core::{
    feeds::FeedsRepository,
    utils::task::{self, Task},
};

pub struct CleanupTask {
    repo: Arc<dyn FeedsRepository>,
}

impl CleanupTask {
    pub fn new(repo: Arc<dyn FeedsRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl Task for CleanupTask {
    async fn run(&self) -> Result<(), task::Error> {
        self.repo
            .cleanup_feeds()
            .await
            .map_err(|e| task::Error(e.into()))?;

        Ok(())
    }
}
