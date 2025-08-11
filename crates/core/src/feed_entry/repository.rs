use chrono::{DateTime, Utc};

use crate::{
    RepositoryError,
    feed::FeedId,
    feed_entry::{FeedEntry, FeedEntryId},
};

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync + 'static {
    async fn find(&self, params: FeedEntryFindParams) -> Result<Vec<FeedEntry>, RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<FeedEntryId>,
    pub feed_id: Option<FeedId>,
    pub cursor: Option<(DateTime<Utc>, FeedEntryId)>,
    pub limit: Option<usize>,
}
