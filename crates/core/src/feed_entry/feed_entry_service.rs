use colette_util::base64;
use uuid::Uuid;

use super::{
    feed_entry_repository::{FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    Cursor, Error, FeedEntry,
};
use crate::common::{IdParams, NonEmptyString, Paginated, PAGINATION_LIMIT};

pub struct FeedEntryService {
    repository: Box<dyn FeedEntryRepository>,
}

impl FeedEntryService {
    pub fn new(repository: impl FeedEntryRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_feed_entries(
        &self,
        query: FeedEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<FeedEntry>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut feed_entries = self
            .repository
            .find(FeedEntryFindParams {
                feed_id: query.feed_id,
                smart_feed_id: query.smart_feed_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id,
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

    pub async fn get_feed_entry(&self, id: Uuid, user_id: Uuid) -> Result<FeedEntry, Error> {
        let mut feed_entries = self
            .repository
            .find(FeedEntryFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if feed_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(feed_entries.swap_remove(0))
    }

    pub async fn update_feed_entry(
        &self,
        id: Uuid,
        data: FeedEntryUpdate,
        user_id: Uuid,
    ) -> Result<FeedEntry, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_feed_entry(id, user_id).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct FeedEntryListQuery {
    pub feed_id: Option<Uuid>,
    pub smart_feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedEntryUpdate {
    pub has_read: Option<bool>,
}

impl From<FeedEntryUpdate> for FeedEntryUpdateData {
    fn from(value: FeedEntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}
