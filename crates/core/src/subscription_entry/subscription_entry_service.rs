use colette_util::base64;
use uuid::Uuid;

use super::{
    Cursor, Error, SubscriptionEntry, SubscriptionEntryFilter, SubscriptionEntryFindParams,
    SubscriptionEntryRepository,
};
use crate::{
    common::{PAGINATION_LIMIT, Paginated},
    stream::{StreamFindParams, StreamRepository},
};

pub struct SubscriptionEntryService {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
    stream_repository: Box<dyn StreamRepository>,
}

impl SubscriptionEntryService {
    pub fn new(
        feed_entry_repository: impl SubscriptionEntryRepository,
        stream_repository: impl StreamRepository,
    ) -> Self {
        Self {
            subscription_entry_repository: Box::new(feed_entry_repository),
            stream_repository: Box::new(stream_repository),
        }
    }

    pub async fn list_subscription_entries(
        &self,
        query: SubscriptionEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<SubscriptionEntry>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut filter = Option::<SubscriptionEntryFilter>::None;
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

        let mut subscription_entries = self
            .subscription_entry_repository
            .find_subscription_entries(SubscriptionEntryFindParams {
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
        if subscription_entries.len() > limit {
            subscription_entries = subscription_entries.into_iter().take(limit).collect();

            if let Some(last) = subscription_entries.last() {
                let c = Cursor {
                    id: last.entry.id,
                    published_at: last.entry.published_at,
                };
                let encoded = base64::encode(&c)?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated {
            data: subscription_entries,
            cursor,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryListQuery {
    pub stream_id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<String>,
}
