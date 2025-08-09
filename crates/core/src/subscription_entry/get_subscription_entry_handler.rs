use uuid::Uuid;

use super::{SubscriptionEntry, SubscriptionEntryFindParams, SubscriptionEntryRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetSubscriptionEntryQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetSubscriptionEntryHandler {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
}

impl GetSubscriptionEntryHandler {
    pub fn new(subscription_entry_repository: impl SubscriptionEntryRepository) -> Self {
        Self {
            subscription_entry_repository: Box::new(subscription_entry_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetSubscriptionEntryQuery> for GetSubscriptionEntryHandler {
    type Response = SubscriptionEntry;
    type Error = GetSubscriptionEntryError;

    async fn handle(
        &self,
        query: GetSubscriptionEntryQuery,
    ) -> Result<Self::Response, Self::Error> {
        let mut subscription_entries = self
            .subscription_entry_repository
            .find(SubscriptionEntryFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if subscription_entries.is_empty() {
            return Err(GetSubscriptionEntryError::NotFound(query.id));
        }

        let subscription_entry = subscription_entries.swap_remove(0);
        if subscription_entry.user_id != query.user_id {
            return Err(GetSubscriptionEntryError::Forbidden(query.id));
        }

        Ok(subscription_entry)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionEntryError {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
