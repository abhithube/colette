use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{SubscriptionEntry, SubscriptionEntryFilter};
use crate::RepositoryError;

#[async_trait::async_trait]
pub trait SubscriptionEntryRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SubscriptionEntryById>, RepositoryError>;

    async fn mark_as_read(&self, id: Uuid) -> Result<(), RepositoryError>;

    async fn mark_as_unread(&self, id: Uuid) -> Result<(), RepositoryError>;
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
