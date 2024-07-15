use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::common::{Paginated, Session, PAGINATION_LIMIT};

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: String,
}

#[derive(Debug)]
pub struct ListEntriesParams {
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<String>,
    pub has_read: Option<bool>,
}

#[async_trait::async_trait]
pub trait EntriesRepository {
    async fn find_many(&self, params: EntryFindManyParams) -> Result<Vec<Entry>, Error>;
}

pub struct EntriesService {
    repo: Arc<dyn EntriesRepository + Send + Sync>,
}

impl EntriesService {
    pub fn new(repo: Arc<dyn EntriesRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListEntriesParams,
        session: Session,
    ) -> Result<Paginated<Entry>, Error> {
        let params = EntryFindManyParams {
            profile_id: session.profile_id,
            limit: (PAGINATION_LIMIT + 1) as i64,
            published_at: params.published_at,
            feed_id: params.feed_id,
            has_read: params.has_read,
        };
        let entries = self.repo.find_many(params).await?;

        let paginated = Paginated::<Entry> {
            has_more: entries.len() > PAGINATION_LIMIT,
            data: entries.into_iter().take(PAGINATION_LIMIT).collect(),
        };

        Ok(paginated)
    }
}

pub struct EntryFindManyParams {
    pub profile_id: String,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<String>,
    pub has_read: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entry not found with id: {0}")]
    NotFound(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
