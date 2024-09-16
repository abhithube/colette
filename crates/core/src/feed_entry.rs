use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_util::DataEncoder;
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

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    pub published_at: DateTime<Utc>,
}

pub struct FeedEntryService {
    repository: Arc<dyn FeedEntryRepository>,
    base64_encoder: Arc<dyn DataEncoder<Cursor>>,
}

impl FeedEntryService {
    pub fn new(
        repository: Arc<dyn FeedEntryRepository>,
        base64_encoder: Arc<dyn DataEncoder<Cursor>>,
    ) -> Self {
        Self {
            repository,
            base64_encoder,
        }
    }

    pub async fn list_feed_entries(
        &self,
        query: FeedEntryListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<FeedEntry>, Error> {
        let cursor = query
            .cursor
            .and_then(|e| self.base64_encoder.decode(&e).ok());

        let mut feed_entries = self
            .repository
            .list(
                profile_id,
                Some(PAGINATION_LIMIT + 1),
                cursor,
                Some(FeedEntryFindManyFilters {
                    feed_id: query.feed_id,
                    has_read: query.has_read,
                    tags: query.tags,
                }),
            )
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if feed_entries.len() > limit {
            feed_entries = feed_entries.into_iter().take(limit).collect();

            if let Some(last) = feed_entries.last() {
                let c = Cursor {
                    id: last.id,
                    published_at: last.published_at,
                };
                let encoded = self.base64_encoder.encode(&c)?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated {
            data: feed_entries,
            cursor,
        })
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
        cursor: Option<Cursor>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Vec<FeedEntry>, Error>;
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
