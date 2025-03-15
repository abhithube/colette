use uuid::Uuid;

use super::{Error, Subscription, SubscriptionFindParams, SubscriptionRepository};
use crate::{
    SubscriptionEntry,
    common::Paginated,
    subscription_entry::{SubscriptionEntryFindParams, SubscriptionEntryRepository},
    tag::TagRepository,
};

pub struct SubscriptionService {
    subscription_repository: Box<dyn SubscriptionRepository>,
    tag_repository: Box<dyn TagRepository>,
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
}

impl SubscriptionService {
    pub fn new(
        subscription_repository: impl SubscriptionRepository,
        tag_repository: impl TagRepository,
        subscription_entry_repository: impl SubscriptionEntryRepository,
    ) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
            tag_repository: Box::new(tag_repository),
            subscription_entry_repository: Box::new(subscription_entry_repository),
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription>, Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
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
            .find(SubscriptionFindParams {
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
        let builder = Subscription::builder()
            .title(data.title)
            .feed_id(data.feed_id)
            .user_id(user_id);

        let subscription = if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_by_ids(ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect::<Vec<_>>();

            builder.tags(tags).build()
        } else {
            builder.build()
        };

        self.subscription_repository
            .save(&subscription, false)
            .await?;

        Ok(subscription)
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        data: SubscriptionUpdate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let Some(mut subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            subscription.title = title;
        }

        if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_by_ids(ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect();

            subscription.tags = Some(tags);
        }

        self.subscription_repository
            .save(&subscription, true)
            .await?;

        Ok(subscription)
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let Some(subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository.delete_by_id(id).await?;

        Ok(())
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
        feed_entry_id: Uuid,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let Some(mut subscription_entry) = self
            .subscription_entry_repository
            .find_by_id(feed_entry_id, subscription_id)
            .await?
        else {
            return Err(Error::NotFound(feed_entry_id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        subscription_entry.has_read = Some(false);

        self.subscription_entry_repository
            .save(&subscription_entry)
            .await?;

        Ok(subscription_entry)
    }

    pub async fn mark_subscription_entry_as_unread(
        &self,
        feed_entry_id: Uuid,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let Some(mut subscription_entry) = self
            .subscription_entry_repository
            .find_by_id(feed_entry_id, subscription_id)
            .await?
        else {
            return Err(Error::NotFound(feed_entry_id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        subscription_entry.has_read = Some(false);

        self.subscription_entry_repository
            .save(&subscription_entry)
            .await?;

        Ok(subscription_entry)
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
