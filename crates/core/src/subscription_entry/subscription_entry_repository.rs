use uuid::Uuid;

use super::{Cursor, Error, SubscriptionEntry, SubscriptionEntryFilter};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait SubscriptionEntryRepository: Send + Sync + 'static {
    async fn find_subscription_entries(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error>;

    async fn find_subscription_entry_by_id(
        &self,
        tx: &dyn Transaction,
        feed_entry_id: Uuid,
    ) -> Result<SubscriptionEntryById, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryById {
    pub feed_entry_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryFindParams {
    pub filter: Option<SubscriptionEntryFilter>,
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}
