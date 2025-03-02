use uuid::Uuid;

use super::{Cursor, Error, FeedEntry};
use crate::common::IdParams;

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find_feed_entries(&self, params: FeedEntryFindParams)
    -> Result<Vec<FeedEntry>, Error>;

    async fn update_feed_entry(
        &self,
        params: IdParams,
        data: FeedEntryUpdateData,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub smart_feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryUpdateData {
    pub has_read: Option<bool>,
}
