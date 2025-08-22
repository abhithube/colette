use url::Url;
use uuid::Uuid;

use crate::{
    auth::UserId,
    common::RepositoryError,
    subscription::{Subscription, SubscriptionDto, SubscriptionId},
    tag::TagId,
};

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<SubscriptionDto>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> Result<Option<Subscription>, RepositoryError>;

    async fn save(&self, data: &Subscription) -> Result<(), RepositoryError>;

    async fn delete_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> Result<(), RepositoryError>;

    async fn import(&self, params: ImportSubscriptionsParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct SubscriptionFindParams {
    pub user_id: UserId,
    pub id: Option<SubscriptionId>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<usize>,
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
