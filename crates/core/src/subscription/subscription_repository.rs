use url::Url;
use uuid::Uuid;

use super::{Error, Subscription};

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn find(&self, params: SubscriptionFindParams) -> Result<Vec<Subscription>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SubscriptionById>, Error>;

    async fn insert(&self, params: SubscriptionInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: SubscriptionUpdateParams) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;

    async fn link_tags(&self, params: SubscriptionLinkTagParams) -> Result<(), Error>;

    async fn import(&self, params: ImportSubscriptionsParams) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<usize>,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SubscriptionInsertParams {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SubscriptionUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionLinkTagParams {
    pub subscription_id: Uuid,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ImportSubscriptionsParams {
    pub subscription_items: Vec<SubscriptionBatchItem>,
    pub tag_titles: Vec<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SubscriptionBatchItem {
    pub feed_url: Url,
    pub feed_link: Url,
    pub feed_title: String,
    pub tag_titles: Vec<String>,
}
