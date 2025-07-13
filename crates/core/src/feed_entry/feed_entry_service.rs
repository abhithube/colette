use std::sync::Arc;

use uuid::Uuid;

use super::{Error, FeedEntry, FeedEntryCursor, FeedEntryParams, FeedEntryRepository};
use crate::pagination::{Paginated, paginate};

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
    ) -> Result<Paginated<FeedEntry, FeedEntryCursor>, Error> {
        let feed_entries = self
            .repository
            .query(FeedEntryParams {
                feed_id: query.feed_id,
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(feed_entries, limit))
        } else {
            Ok(Paginated::default())
        }
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
    pub cursor: Option<FeedEntryCursor>,
    pub limit: Option<usize>,
}
