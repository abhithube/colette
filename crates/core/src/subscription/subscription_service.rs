use uuid::Uuid;

use super::{
    Error, Subscription, SubscriptionCreateData, SubscriptionEntryUpdateData,
    SubscriptionEntryUpdateParams, SubscriptionFindParams, SubscriptionRepository,
    SubscriptionUpdateData,
};
use crate::{
    SubscriptionEntry,
    common::{Paginated, TransactionManager},
    subscription_entry::{SubscriptionEntryFindParams, SubscriptionEntryRepository},
};

pub struct SubscriptionService {
    subscription_repository: Box<dyn SubscriptionRepository>,
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl SubscriptionService {
    pub fn new(
        subscription_repository: impl SubscriptionRepository,
        subscription_entry_repository: impl SubscriptionEntryRepository,
        tx_manager: impl TransactionManager,
    ) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
            subscription_entry_repository: Box::new(subscription_entry_repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription>, Error> {
        let feeds = self
            .subscription_repository
            .find_subscriptions(SubscriptionFindParams {
                tags: query.tags,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: feeds,
            cursor: None,
        })
    }

    pub async fn get_subscription(&self, id: Uuid, user_id: Uuid) -> Result<Subscription, Error> {
        let mut feeds = self
            .subscription_repository
            .find_subscriptions(SubscriptionFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        let feed = feeds.swap_remove(0);
        if feed.user_id != user_id {
            return Err(Error::Forbidden(feed.id));
        }

        Ok(feed)
    }

    pub async fn create_subscription(
        &self,
        data: SubscriptionCreate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let id = self
            .subscription_repository
            .create_subscription(SubscriptionCreateData {
                title: data.title,
                feed_id: data.feed_id,
                tags: data.tags,
                user_id,
            })
            .await?;

        self.get_subscription(id, user_id).await
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        data: SubscriptionUpdate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let tx = self.tx_manager.begin().await?;

        let feed = self
            .subscription_repository
            .find_subscription_by_id(&*tx, id)
            .await?;
        if feed.user_id != user_id {
            return Err(Error::Forbidden(feed.id));
        }

        self.subscription_repository
            .update_subscription(&*tx, feed.id, data.into())
            .await?;

        tx.commit().await?;

        self.get_subscription(feed.id, feed.user_id).await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let feed = self
            .subscription_repository
            .find_subscription_by_id(&*tx, id)
            .await?;
        if feed.user_id != user_id {
            return Err(Error::Forbidden(feed.id));
        }

        self.subscription_repository
            .delete_subscription(&*tx, feed.id)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_subscription_entry(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let mut feed_entries = self
            .subscription_entry_repository
            .find_subscription_entries(SubscriptionEntryFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feed_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        let feed_entry = feed_entries.swap_remove(0);
        if feed_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry.entry.id));
        }

        Ok(feed_entry)
    }

    pub async fn mark_subscription_entry_as_read(
        &self,
        feed_entry_id: Uuid,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let tx = self.tx_manager.begin().await?;

        let subscription_entry = self
            .subscription_entry_repository
            .find_subscription_entry_by_id(&*tx, feed_entry_id)
            .await?;
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(subscription_entry.feed_entry_id));
        }

        self.subscription_repository
            .update_subscription_entry(
                &*tx,
                SubscriptionEntryUpdateParams {
                    feed_entry_id,
                    subscription_id,
                    user_id,
                },
                SubscriptionEntryUpdateData { has_read: true },
            )
            .await?;

        tx.commit().await?;

        self.get_subscription_entry(subscription_entry.feed_entry_id, subscription_entry.user_id)
            .await
    }

    pub async fn mark_subscription_entry_as_unread(
        &self,
        feed_entry_id: Uuid,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let tx = self.tx_manager.begin().await?;

        let subscription_entry = self
            .subscription_entry_repository
            .find_subscription_entry_by_id(&*tx, feed_entry_id)
            .await?;
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(subscription_entry.feed_entry_id));
        }

        self.subscription_repository
            .update_subscription_entry(
                &*tx,
                SubscriptionEntryUpdateParams {
                    feed_entry_id,
                    subscription_id,
                    user_id,
                },
                SubscriptionEntryUpdateData { has_read: false },
            )
            .await?;

        tx.commit().await?;

        self.get_subscription_entry(subscription_entry.feed_entry_id, subscription_entry.user_id)
            .await
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionListQuery {
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreate {
    pub title: String,
    pub feed_id: Uuid,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionUpdate {
    pub title: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

impl From<SubscriptionUpdate> for SubscriptionUpdateData {
    fn from(value: SubscriptionUpdate) -> Self {
        Self {
            title: value.title,
            tags: value.tags,
        }
    }
}
