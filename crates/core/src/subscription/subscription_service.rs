use uuid::Uuid;

use super::{
    Error, Subscription, SubscriptionCreateParams, SubscriptionDeleteParams,
    SubscriptionEntryUpdateParams, SubscriptionFindByIdParams, SubscriptionFindParams,
    SubscriptionRepository, SubscriptionTagsLinkParams, SubscriptionUpdateParams,
};
use crate::{
    SubscriptionEntry,
    common::{Paginated, TransactionManager},
    subscription_entry::{
        SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams, SubscriptionEntryRepository,
    },
    tag::{TagFindByIdsParams, TagRepository},
};

pub struct SubscriptionService {
    subscription_repository: Box<dyn SubscriptionRepository>,
    tag_repository: Box<dyn TagRepository>,
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl SubscriptionService {
    pub fn new(
        subscription_repository: impl SubscriptionRepository,
        tag_repository: impl TagRepository,
        subscription_entry_repository: impl SubscriptionEntryRepository,
        tx_manager: impl TransactionManager,
    ) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
            tag_repository: Box::new(tag_repository),
            subscription_entry_repository: Box::new(subscription_entry_repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription>, Error> {
        let subscriptions = self
            .subscription_repository
            .find_subscriptions(SubscriptionFindParams {
                tags: query.tags,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: subscriptions,
            cursor: None,
        })
    }

    pub async fn get_subscription(&self, id: Uuid, user_id: Uuid) -> Result<Subscription, Error> {
        let mut subscriptions = self
            .subscription_repository
            .find_subscriptions(SubscriptionFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscriptions.is_empty() {
            return Err(Error::NotFound(id));
        }

        let subscription = subscriptions.swap_remove(0);
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(subscription.id));
        }

        Ok(subscription)
    }

    pub async fn create_subscription(
        &self,
        data: SubscriptionCreate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let tx = self.tx_manager.begin().await?;

        let id = Uuid::new_v4();

        self.subscription_repository
            .create_subscription(
                &*tx,
                SubscriptionCreateParams {
                    id,
                    title: data.title,
                    feed_id: data.feed_id,
                    user_id,
                },
            )
            .await?;

        if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_tags_by_ids(&*tx, TagFindByIdsParams { ids })
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect();

            self.subscription_repository
                .link_tags(
                    &*tx,
                    SubscriptionTagsLinkParams {
                        subscription_id: id,
                        tags,
                    },
                )
                .await?;
        }

        tx.commit().await?;

        self.get_subscription(id, user_id).await
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        data: SubscriptionUpdate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let tx = self.tx_manager.begin().await?;

        let subscription = self
            .subscription_repository
            .find_subscription_by_id(&*tx, SubscriptionFindByIdParams { id })
            .await?;
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository
            .update_subscription(
                &*tx,
                SubscriptionUpdateParams {
                    id,
                    title: data.title,
                },
            )
            .await?;

        if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_tags_by_ids(&*tx, TagFindByIdsParams { ids })
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect();

            self.subscription_repository
                .link_tags(
                    &*tx,
                    SubscriptionTagsLinkParams {
                        subscription_id: id,
                        tags,
                    },
                )
                .await?;
        }

        tx.commit().await?;

        self.get_subscription(id, user_id).await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let subscription = self
            .subscription_repository
            .find_subscription_by_id(&*tx, SubscriptionFindByIdParams { id })
            .await?;
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository
            .delete_subscription(&*tx, SubscriptionDeleteParams { id })
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_subscription_entry(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let mut subscription_entries = self
            .subscription_entry_repository
            .find_subscription_entries(SubscriptionEntryFindParams {
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
        feed_entry_id: Uuid,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let tx = self.tx_manager.begin().await?;

        let subscription_entry = self
            .subscription_entry_repository
            .find_subscription_entry_by_id(&*tx, SubscriptionEntryFindByIdParams { feed_entry_id })
            .await?;
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        self.subscription_repository
            .update_subscription_entry(
                &*tx,
                SubscriptionEntryUpdateParams {
                    feed_entry_id,
                    subscription_id,
                    user_id,
                    has_read: true,
                },
            )
            .await?;

        tx.commit().await?;

        self.get_subscription_entry(feed_entry_id, user_id).await
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
            .find_subscription_entry_by_id(&*tx, SubscriptionEntryFindByIdParams { feed_entry_id })
            .await?;
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        self.subscription_repository
            .update_subscription_entry(
                &*tx,
                SubscriptionEntryUpdateParams {
                    feed_entry_id,
                    subscription_id,
                    user_id,
                    has_read: false,
                },
            )
            .await?;

        tx.commit().await?;

        self.get_subscription_entry(feed_entry_id, user_id).await
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
