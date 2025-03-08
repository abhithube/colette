use uuid::Uuid;

use super::{Cursor, Error, FeedEntry};

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find_feed_entries(&self, params: FeedEntryFindParams)
    -> Result<Vec<FeedEntry>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}
