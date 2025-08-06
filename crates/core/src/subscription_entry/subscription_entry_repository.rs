use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Error, SubscriptionEntry, SubscriptionEntryFilter};

#[async_trait::async_trait]
pub trait SubscriptionEntryRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SubscriptionEntryById>, Error>;

    async fn mark_as_read(&self, id: Uuid) -> Result<(), Error>;

    async fn mark_as_unread(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryFindParams {
    pub filter: Option<SubscriptionEntryFilter>,
    pub id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
    pub with_feed_entry: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionEntryById {
    pub id: Uuid,
    pub user_id: Uuid,
}
