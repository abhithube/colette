use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Error, FeedEntry};

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find(&self, params: FeedEntryFindParams) -> Result<Vec<FeedEntry>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
