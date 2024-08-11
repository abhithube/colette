use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{Paginated, Session, PAGINATION_LIMIT};

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

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UpdateEntry {
    pub has_read: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ListEntriesParams {
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub cursor: Option<String>,
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

pub struct EntriesService {
    repo: Arc<dyn EntriesRepository>,
}

impl EntriesService {
    pub fn new(repo: Arc<dyn EntriesRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListEntriesParams,
        session: Session,
    ) -> Result<Paginated<Entry>, Error> {
        self.repo
            .find_many_entries(
                session.profile_id,
                Some((PAGINATION_LIMIT + 1) as u64),
                params.cursor,
                Some(EntriesFindManyFilters {
                    feed_id: params.feed_id,
                    has_read: params.has_read,
                    tags: params.tags,
                }),
            )
            .await
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Entry, Error> {
        self.repo.find_one_entry(id, session.profile_id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateEntry,
        session: Session,
    ) -> Result<Entry, Error> {
        self.repo
            .update_entry(id, session.profile_id, data.into())
            .await
    }
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

impl From<UpdateEntry> for EntriesUpdateData {
    fn from(value: UpdateEntry) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entry not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
