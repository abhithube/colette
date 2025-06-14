use uuid::Uuid;

use super::{
    Cursor, Error, SubscriptionEntry, SubscriptionEntryFilter, SubscriptionEntryParams,
    SubscriptionEntryRepository,
};
use crate::{
    common::{PAGINATION_LIMIT, Paginated},
    stream::{StreamParams, StreamRepository},
};

pub struct SubscriptionEntryService {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
    stream_repository: Box<dyn StreamRepository>,
}

impl SubscriptionEntryService {
    pub fn new(
        subscription_entry_repository: impl SubscriptionEntryRepository,
        stream_repository: impl StreamRepository,
    ) -> Self {
        Self {
            subscription_entry_repository: Box::new(subscription_entry_repository),
            stream_repository: Box::new(stream_repository),
        }
    }

    pub async fn list_subscription_entries(
        &self,
        query: SubscriptionEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<SubscriptionEntry>, Error> {
        let cursor = query
            .cursor
            .and_then(|e| colette_util::base64_decode::<Cursor>(&e).ok());

        let mut filter = Option::<SubscriptionEntryFilter>::None;
        if let Some(stream_id) = query.stream_id {
            let mut streams = self
                .stream_repository
                .query(StreamParams {
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
            .query(SubscriptionEntryParams {
                filter,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id: Some(user_id),
                cursor: cursor.map(|e| (e.published_at, e.id)),
                limit: Some(PAGINATION_LIMIT + 1),
                with_read_entry: true,
                ..Default::default()
            })
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if subscription_entries.len() > limit {
            subscription_entries = subscription_entries.into_iter().take(limit).collect();

            if let Some(last) = subscription_entries.last()
                && let Some(ref entry) = last.feed_entry
            {
                let c = Cursor {
                    published_at: entry.published_at,
                    id: entry.id,
                };
                let encoded = colette_util::base64_encode(&c)?;

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
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<String>,
}
