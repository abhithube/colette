use std::sync::Arc;

use uuid::Uuid;

use super::{
    Error, SubscriptionEntry, SubscriptionEntryCursor, SubscriptionEntryFilter,
    SubscriptionEntryParams, SubscriptionEntryRepository,
};
use crate::{
    collection::{CollectionParams, CollectionRepository},
    pagination::{Paginated, paginate},
};

pub struct SubscriptionEntryService {
    subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
    collection_repository: Arc<dyn CollectionRepository>,
}

impl SubscriptionEntryService {
    pub fn new(
        subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
        collection_repository: Arc<dyn CollectionRepository>,
    ) -> Self {
        Self {
            subscription_entry_repository,
            collection_repository,
        }
    }

    pub async fn list_subscription_entries(
        &self,
        query: SubscriptionEntryListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<SubscriptionEntry, SubscriptionEntryCursor>, Error> {
        let filter = Option::<SubscriptionEntryFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let collections = self
                .collection_repository
                .query(CollectionParams {
                    id: Some(collection_id),
                    user_id: Some(user_id),
                    ..Default::default()
                })
                .await?;
            if collections.is_empty() {
                return Ok(Paginated {
                    items: Default::default(),
                    cursor: None,
                });
            }

            // filter = Some(collections.swap_remove(0).filter);
        }

        let subscription_entries = self
            .subscription_entry_repository
            .query(SubscriptionEntryParams {
                filter,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                with_read_entry: true,
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(subscription_entries, limit))
        } else {
            Ok(Paginated {
                items: subscription_entries,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryListQuery {
    pub collection_id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<SubscriptionEntryCursor>,
    pub limit: Option<usize>,
}
