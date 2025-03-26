use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Error, SubscriptionEntry, SubscriptionEntryFilter};

#[async_trait::async_trait]
pub trait SubscriptionEntryRepository: Send + Sync + 'static {
    async fn query(&self, params: SubscriptionEntryParams)
    -> Result<Vec<SubscriptionEntry>, Error>;

    async fn find_by_id(
        &self,
        feed_entry_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Option<SubscriptionEntry>, Error> {
        Ok(self
            .query(SubscriptionEntryParams {
                feed_entry_id: Some(feed_entry_id),
                subscription_id: Some(subscription_id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionEntryParams {
    pub filter: Option<SubscriptionEntryFilter>,
    pub feed_entry_id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<String>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<u64>,
    pub with_read_entries: bool,
}
