use std::sync::Arc;

use uuid::Uuid;

use super::{
    Error, SubscriptionEntry, SubscriptionEntryCursor, SubscriptionEntryFilter,
    SubscriptionEntryFindParams, SubscriptionEntryRepository,
};
use crate::{
    collection::{CollectionFindParams, CollectionRepository},
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
                .find(CollectionFindParams {
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
            .find(SubscriptionEntryFindParams {
                filter,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                with_feed_entry: true,
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

    pub async fn get_subscription_entry(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let mut subscription_entries = self
            .subscription_entry_repository
            .find(SubscriptionEntryFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscription_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        let subscription_entry = subscription_entries.swap_remove(0);
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        Ok(subscription_entry)
    }

    pub async fn mark_subscription_entry_as_read(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let Some(subscription_entry) = self.subscription_entry_repository.find_by_id(id).await?
        else {
            return Err(Error::NotFound(id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_entry_repository.mark_as_read(id).await?;

        Ok(())
    }

    pub async fn mark_subscription_entry_as_unread(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let Some(subscription_entry) = self.subscription_entry_repository.find_by_id(id).await?
        else {
            return Err(Error::NotFound(id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_entry_repository
            .mark_as_unread(id)
            .await?;

        Ok(())
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
