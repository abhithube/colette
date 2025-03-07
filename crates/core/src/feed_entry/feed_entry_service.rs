use colette_util::base64;
use uuid::Uuid;

use super::{Cursor, Error, FeedEntry, FeedEntryFindParams, FeedEntryRepository};
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
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut feed_entries = self
            .feed_entry_repository
            .find_feed_entries(FeedEntryFindParams {
                feed_id: query.feed_id,
                has_read: query.has_read,
                tags: query.tags,
                limit: Some(PAGINATION_LIMIT as i64 + 1),
                cursor,
                ..Default::default()
            })
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
                let encoded = base64::encode(&c)?;

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
            .find_feed_entries(FeedEntryFindParams {
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
    pub stream_id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<String>,
}
