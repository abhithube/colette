use std::sync::Arc;

use uuid::Uuid;

use super::{
    Error, SubscriptionEntry, SubscriptionEntryCursor, SubscriptionEntryFilter,
    SubscriptionEntryParams, SubscriptionEntryRepository,
};
use crate::{
    common::{PAGINATION_LIMIT, Paginated, Paginator},
    stream::{StreamParams, StreamRepository},
};

pub struct SubscriptionEntryService {
    subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
    stream_repository: Arc<dyn StreamRepository>,
}

impl SubscriptionEntryService {
    pub fn new(
        subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
        stream_repository: Arc<dyn StreamRepository>,
    ) -> Self {
        Self {
            subscription_entry_repository,
            stream_repository,
        }
    }

    pub async fn list_subscription_entries(
        &self,
        query: SubscriptionEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<SubscriptionEntry>, Error> {
        let cursor = query
            .cursor
            .map(|e| Paginator::decode_cursor::<SubscriptionEntryCursor>(&e))
            .transpose()?;

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
                    items: Default::default(),
                    cursor: None,
                });
            }

            filter = Some(streams.swap_remove(0).filter);
        }

        let subscription_entries = self
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

        let data = Paginator::paginate(subscription_entries, PAGINATION_LIMIT)?;

        Ok(data)
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
