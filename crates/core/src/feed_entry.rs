use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{IdParams, Paginated};

#[derive(Clone, Debug, serde::Serialize)]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: Uuid,
}

#[async_trait::async_trait]
pub trait FeedEntryRepository: Send + Sync {
    async fn find_many(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Paginated<FeedEntry>, Error>;

    async fn find_one(&self, params: IdParams) -> Result<FeedEntry, Error>;

    async fn update(&self, params: IdParams, data: FeedEntryUpdateData)
        -> Result<FeedEntry, Error>;
}

#[derive(Clone, Debug)]
pub struct FeedEntryFindManyFilters {
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct FeedEntryUpdateData {
    pub has_read: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
