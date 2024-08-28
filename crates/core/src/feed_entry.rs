use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{Findable, IdParams, Paginated, Updatable, PAGINATION_LIMIT};

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

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryUpdate {
    pub has_read: Option<bool>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryListQuery {
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub cursor: Option<String>,
}

pub struct FeedEntryService {
    repository: Arc<dyn FeedEntryRepository>,
}

impl FeedEntryService {
    pub fn new(repository: Arc<dyn FeedEntryRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_feed_entries(
        &self,
        query: FeedEntryListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<FeedEntry>, Error> {
        self.repository
            .list(
                profile_id,
                Some(PAGINATION_LIMIT),
                query.cursor,
                Some(FeedEntryFindManyFilters {
                    feed_id: query.feed_id,
                    has_read: query.has_read,
                    tags: query.tags,
                }),
            )
            .await
    }

    pub async fn get_feed_entry(&self, id: Uuid, profile_id: Uuid) -> Result<FeedEntry, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn update_feed_entry(
        &self,
        id: Uuid,
        data: FeedEntryUpdate,
        profile_id: Uuid,
    ) -> Result<FeedEntry, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }
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

impl From<FeedEntryUpdate> for FeedEntryUpdateData {
    fn from(value: FeedEntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
