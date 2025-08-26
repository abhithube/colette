use colette_authentication::UserId;
use colette_common::RepositoryError;
use url::Url;

use crate::{Subscription, SubscriptionId};

pub trait SubscriptionRepository: Sync {
    fn find_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> impl std::future::Future<Output = Result<Option<Subscription>, RepositoryError>> + Send;

    fn save(
        &self,
        data: &Subscription,
    ) -> impl std::future::Future<Output = Result<(), RepositoryError>> + Send;

    fn delete_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> impl std::future::Future<Output = Result<(), RepositoryError>> + Send;

    fn import(
        &self,
        params: ImportSubscriptionsParams,
    ) -> impl std::future::Future<Output = Result<(), RepositoryError>> + Send;
}

#[derive(Debug, Clone)]
pub struct ImportSubscriptionsParams {
    pub subscription_items: Vec<SubscriptionBatchItem>,
    pub tag_titles: Vec<String>,
    pub feed_refresh_interval: u32,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct SubscriptionBatchItem {
    pub feed_url: Url,
    pub feed_link: Url,
    pub feed_title: String,
    pub tag_titles: Vec<String>,
}
