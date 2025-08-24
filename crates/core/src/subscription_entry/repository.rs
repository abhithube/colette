use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    auth::UserId,
    common::RepositoryError,
    subscription::SubscriptionId,
    subscription_entry::{SubscriptionEntry, SubscriptionEntryFilter, SubscriptionEntryId},
    tag::TagId,
};

#[async_trait::async_trait]
pub trait SubscriptionEntryRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: SubscriptionEntryId,
    ) -> Result<Option<SubscriptionEntry>, RepositoryError> {
        let mut subscription_entries = self
            .find(SubscriptionEntryFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscription_entries.is_empty() {
            return Ok(None);
        }

        Ok(Some(subscription_entries.swap_remove(0)))
    }

    async fn mark_as_read(&self, id: SubscriptionEntryId) -> Result<(), RepositoryError>;

    async fn mark_as_unread(&self, id: SubscriptionEntryId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryFindParams {
    pub filter: Option<SubscriptionEntryFilter>,
    pub id: Option<SubscriptionEntryId>,
    pub subscription_id: Option<SubscriptionId>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<TagId>>,
    pub user_id: Option<UserId>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
    pub with_feed_entry: bool,
}
