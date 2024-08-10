use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{FindOneParams, Paginated, Session, PAGINATION_LIMIT};

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
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait EntriesRepository: Send + Sync {
    async fn find_many_entries(&self, params: EntriesFindManyParams) -> Result<Vec<Entry>, Error>;

    async fn find_one_entry(&self, params: FindOneParams) -> Result<Entry, Error>;

    async fn update_entry(
        &self,
        params: FindOneParams,
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
        let entries = self
            .repo
            .find_many_entries(EntriesFindManyParams {
                profile_id: session.profile_id,
                limit: (PAGINATION_LIMIT + 1) as u64,
                published_at: params.published_at,
                feed_id: params.feed_id,
                has_read: params.has_read,
                tags: params.tags,
            })
            .await?;

        Ok(Paginated::<Entry> {
            has_more: entries.len() > PAGINATION_LIMIT,
            data: entries.into_iter().take(PAGINATION_LIMIT).collect(),
        })
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Entry, Error> {
        self.repo
            .find_one_entry(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateEntry,
        session: Session,
    ) -> Result<Entry, Error> {
        self.repo
            .update_entry(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await
    }
}

#[derive(Clone, Debug)]
pub struct EntriesFindManyParams {
    pub profile_id: Uuid,
    pub limit: u64,
    pub published_at: Option<DateTime<Utc>>,
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
