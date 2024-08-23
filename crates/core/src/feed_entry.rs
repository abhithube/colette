use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{Findable, IdParams, Paginated, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: Uuid,
}

#[async_trait::async_trait]
pub trait FeedEntryRepository:
    Findable<Params = IdParams, Output = Result<FeedEntry, Error>>
    + Updatable<Params = IdParams, Data = FeedEntryUpdateData, Output = Result<FeedEntry, Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Paginated<FeedEntry>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct FeedEntryFindManyFilters {
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
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
