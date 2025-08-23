use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    subscription_entry::{
        SubscriptionEntry, SubscriptionEntryError, SubscriptionEntryFindParams,
        SubscriptionEntryId, SubscriptionEntryRepository,
    },
};

#[derive(Debug, Clone)]
pub struct GetSubscriptionEntryQuery {
    pub id: SubscriptionEntryId,
    pub user_id: UserId,
}

pub struct GetSubscriptionEntryHandler<SER: SubscriptionEntryRepository> {
    subscription_entry_repository: SER,
}

impl<SER: SubscriptionEntryRepository> GetSubscriptionEntryHandler<SER> {
    pub fn new(subscription_entry_repository: SER) -> Self {
        Self {
            subscription_entry_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SER: SubscriptionEntryRepository> Handler<GetSubscriptionEntryQuery>
    for GetSubscriptionEntryHandler<SER>
{
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
