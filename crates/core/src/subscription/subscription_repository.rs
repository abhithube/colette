use uuid::Uuid;

use super::{Cursor, Error, Subscription};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn find_subscriptions(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<Subscription>, Error>;

    async fn find_subscription_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<SubscriptionById, Error>;

    async fn create_subscription(&self, data: SubscriptionCreateData) -> Result<Uuid, Error>;

    async fn update_subscription(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: SubscriptionUpdateData,
    ) -> Result<(), Error>;

    async fn delete_subscription(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error>;

    async fn update_subscription_entry(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryUpdateParams,
        data: SubscriptionEntryUpdateData,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionFindParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreateData {
    pub title: String,
    pub feed_id: Uuid,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionUpdateData {
    pub title: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryUpdateParams {
    pub feed_entry_id: Uuid,
    pub subscription_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryUpdateData {
    pub has_read: bool,
}
