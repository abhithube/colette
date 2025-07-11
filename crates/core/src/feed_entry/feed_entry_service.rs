use std::sync::Arc;

use uuid::Uuid;

use super::{Error, FeedEntry, FeedEntryCursor, FeedEntryParams, FeedEntryRepository};
use crate::common::{PAGINATION_LIMIT, Paginated, Paginator};

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
    ) -> Result<Paginated<FeedEntry>, Error> {
        let cursor = query
            .cursor
            .map(|e| Paginator::decode_cursor::<FeedEntryCursor>(&e))
            .transpose()?;

        let feed_entries = self
            .repository
            .query(FeedEntryParams {
                feed_id: query.feed_id,
                cursor: cursor.map(|e| (e.published_at, e.id)),
                limit: Some(PAGINATION_LIMIT + 1),
                ..Default::default()
            })
            .await?;

        let data = Paginator::paginate(feed_entries, PAGINATION_LIMIT)?;

        Ok(data)
    }

    pub async fn get_feed_entry(&self, id: Uuid) -> Result<FeedEntry, Error> {
        let mut feed_entries = self
            .repository
            .query(FeedEntryParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feed_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(feed_entries.swap_remove(0))
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryListQuery {
    pub feed_id: Option<Uuid>,
    pub cursor: Option<String>,
}
