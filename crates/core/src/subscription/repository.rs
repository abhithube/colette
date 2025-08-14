use url::Url;

use crate::{
    common::RepositoryError,
    feed::FeedId,
    subscription::{Subscription, SubscriptionId},
    tag::TagId,
    auth::UserId,
};

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<Subscription>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<Subscription>, RepositoryError> {
        let mut subscriptions = self
            .find(SubscriptionFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscriptions.is_empty() {
            return Ok(None);
        }

        Ok(Some(subscriptions.swap_remove(0)))
    }

    async fn insert(
        &self,
        params: SubscriptionInsertParams,
    ) -> Result<SubscriptionId, RepositoryError>;

    async fn update(&self, params: SubscriptionUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: SubscriptionId) -> Result<(), RepositoryError>;

    async fn link_tags(&self, params: SubscriptionLinkTagParams) -> Result<(), RepositoryError>;

    async fn import(&self, params: ImportSubscriptionsParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionFindParams {
    pub id: Option<SubscriptionId>,
    pub user_id: Option<UserId>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<(String, SubscriptionId)>,
    pub limit: Option<usize>,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionInsertParams {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: FeedId,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct SubscriptionUpdateParams {
    pub id: SubscriptionId,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionLinkTagParams {
    pub subscription_id: SubscriptionId,
    pub tag_ids: Vec<TagId>,
}

#[derive(Debug, Clone)]
pub struct ImportSubscriptionsParams {
    pub subscription_items: Vec<SubscriptionBatchItem>,
    pub tag_titles: Vec<String>,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct SubscriptionBatchItem {
    pub feed_url: Url,
    pub feed_link: Url,
    pub feed_title: String,
    pub tag_titles: Vec<String>,
}
