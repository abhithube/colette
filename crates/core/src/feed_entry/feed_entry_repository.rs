use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Error, FeedEntry};

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn query(&self, params: FeedEntryParams) -> Result<Vec<FeedEntry>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
