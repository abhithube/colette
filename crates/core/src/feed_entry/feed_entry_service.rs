use colette_util::base64;
use uuid::Uuid;

use super::{
    Cursor, Error, FeedEntry, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData,
};
use crate::{
    common::{PAGINATION_LIMIT, Paginated, TransactionManager},
    feed_entry::FeedEntryFilter,
    stream::{StreamFindParams, StreamRepository},
};

pub struct FeedEntryService {
    feed_entry_repository: Box<dyn FeedEntryRepository>,
    stream_repository: Box<dyn StreamRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl FeedEntryService {
    pub fn new(
        feed_entry_repository: impl FeedEntryRepository,
        stream_repository: impl StreamRepository,
        tx_manager: impl TransactionManager,
    ) -> Self {
        Self {
            feed_entry_repository: Box::new(feed_entry_repository),
            stream_repository: Box::new(stream_repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_feed_entries(
        &self,
        query: FeedEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<FeedEntry>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut filter = Option::<FeedEntryFilter>::None;
        if let Some(stream_id) = query.stream_id {
            let mut streams = self
                .stream_repository
                .find_streams(StreamFindParams {
                    id: Some(stream_id),
                    user_id: Some(user_id),
                    ..Default::default()
                })
                .await?;
            if streams.is_empty() {
                return Ok(Paginated {
                    data: Default::default(),
                    cursor: None,
                });
            }

            filter = Some(streams.swap_remove(0).filter);
        }

        let mut feed_entries = self
            .feed_entry_repository
            .find_feed_entries(FeedEntryFindParams {
                filter,
                feed_id: query.feed_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id: Some(user_id),
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
            .feed_entry_repository
            .find_feed_entries(FeedEntryFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feed_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        let feed_entry = feed_entries.swap_remove(0);
        if feed_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry.id));
        }

        Ok(feed_entry)
    }

    pub async fn update_feed_entry(
        &self,
        id: Uuid,
        data: FeedEntryUpdate,
        user_id: Uuid,
    ) -> Result<FeedEntry, Error> {
        let tx = self.tx_manager.begin().await?;

        let feed_entry = self
            .feed_entry_repository
            .find_feed_entry_by_id(&*tx, id)
            .await?;
        if feed_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry.id));
        }

        self.feed_entry_repository
            .update_feed_entry(&*tx, feed_entry.id, data.into())
            .await?;

        tx.commit().await?;

        self.get_feed_entry(feed_entry.id, feed_entry.user_id).await
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

#[derive(Debug, Clone, Default)]
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
