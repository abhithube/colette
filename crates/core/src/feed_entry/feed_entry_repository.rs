use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::FeedEntry;
use crate::RepositoryError;

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find(&self, params: FeedEntryFindParams) -> Result<Vec<FeedEntry>, RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
