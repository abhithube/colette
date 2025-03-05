use uuid::Uuid;

use super::{
    Error, Subscription, SubscriptionCreateData, SubscriptionFindParams, SubscriptionRepository,
    SubscriptionUpdateData,
};
use crate::common::{Paginated, TransactionManager};

pub struct SubscriptionService {
    repository: Box<dyn SubscriptionRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl SubscriptionService {
    pub fn new(
        repository: impl SubscriptionRepository,
        tx_manager: impl TransactionManager,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription>, Error> {
        let feeds = self
            .repository
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
            .repository
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
            .repository
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

        let feed = self.repository.find_subscription_by_id(&*tx, id).await?;
        if feed.user_id != user_id {
            return Err(Error::Forbidden(feed.id));
        }

        self.repository
            .update_subscription(&*tx, feed.id, data.into())
            .await?;

        tx.commit().await?;

        self.get_subscription(feed.id, feed.user_id).await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let feed = self.repository.find_subscription_by_id(&*tx, id).await?;
        if feed.user_id != user_id {
            return Err(Error::Forbidden(feed.id));
        }

        self.repository.delete_subscription(&*tx, feed.id).await?;

        tx.commit().await?;

        Ok(())
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
