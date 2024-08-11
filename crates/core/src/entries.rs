use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::Paginated;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
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
pub trait EntriesRepository: Send + Sync {
    async fn find_many_entries(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<EntriesFindManyFilters>,
    ) -> Result<Paginated<Entry>, Error>;

    async fn find_one_entry(&self, id: Uuid, profile_id: Uuid) -> Result<Entry, Error>;

    async fn update_entry(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: EntriesUpdateData,
    ) -> Result<Entry, Error>;
}

#[derive(Clone, Debug)]
pub struct EntriesFindManyFilters {
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct EntriesUpdateData {
    pub has_read: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entry not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
