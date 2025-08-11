use crate::{
    Handler, RepositoryError,
    subscription_entry::{
        SubscriptionEntry, SubscriptionEntryError, SubscriptionEntryFindParams,
        SubscriptionEntryId, SubscriptionEntryRepository,
    },
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct GetSubscriptionEntryQuery {
    pub id: SubscriptionEntryId,
    pub user_id: UserId,
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
        subscription_entry.authorize(query.user_id)?;

        Ok(subscription_entry)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionEntryError {
    #[error("feed entry not found with ID: {0}")]
    NotFound(SubscriptionEntryId),

    #[error(transparent)]
    Core(#[from] SubscriptionEntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
