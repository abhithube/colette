use uuid::Uuid;

use super::{Cursor, Error, FeedEntry, FeedEntryParams, FeedEntryRepository};
use crate::common::{PAGINATION_LIMIT, Paginated};

pub struct FeedEntryService {
    feed_entry_repository: Box<dyn FeedEntryRepository>,
}

impl FeedEntryService {
    pub fn new(feed_entry_repository: impl FeedEntryRepository) -> Self {
        Self {
            feed_entry_repository: Box::new(feed_entry_repository),
        }
    }

    pub async fn list_feed_entries(
        &self,
        query: FeedEntryListQuery,
    ) -> Result<Paginated<FeedEntry>, Error> {
        let cursor = query
            .cursor
            .and_then(|e| colette_util::base64_decode::<Cursor>(&e).ok());

        let mut feed_entries = self
            .feed_entry_repository
            .query(FeedEntryParams {
                feed_id: query.feed_id,
                cursor: cursor.map(|e| (e.published_at, e.id)),
                limit: Some(PAGINATION_LIMIT + 1),
                ..Default::default()
            })
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if feed_entries.len() > limit {
            feed_entries = feed_entries.into_iter().take(limit).collect();

            if let Some(last) = feed_entries.last() {
                let c = Cursor {
                    published_at: last.published_at,
                    id: last.id,
                };
                let encoded = colette_util::base64_encode(&c)?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated {
            data: feed_entries,
            cursor,
        })
    }

    pub async fn get_feed_entry(&self, id: Uuid) -> Result<FeedEntry, Error> {
        let mut feed_entries = self
            .feed_entry_repository
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
