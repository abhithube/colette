use uuid::Uuid;

use super::{Cursor, Error, FeedEntry};
use crate::{common::Transaction, feed_entry::FeedEntryFilter};

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find_feed_entries(&self, params: FeedEntryFindParams)
    -> Result<Vec<FeedEntry>, Error>;

    async fn find_feed_entry_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<FeedEntryById, Error>;

    async fn update_feed_entry(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: FeedEntryUpdateData,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub filter: Option<FeedEntryFilter>,
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryUpdateData {
    pub has_read: Option<bool>,
}
