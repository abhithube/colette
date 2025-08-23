use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    subscription_entry::{
        SubscriptionEntryError, SubscriptionEntryId, SubscriptionEntryRepository,
    },
};

#[derive(Debug, Clone)]
pub struct MarkSubscriptionEntryAsReadCommand {
    pub id: SubscriptionEntryId,
    pub user_id: UserId,
}

pub struct MarkSubscriptionEntryAsReadHandler<SER: SubscriptionEntryRepository> {
    subscription_entry_repository: SER,
}

impl<SER: SubscriptionEntryRepository> MarkSubscriptionEntryAsReadHandler<SER> {
    pub fn new(subscription_entry_repository: SER) -> Self {
        Self {
            subscription_entry_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SER: SubscriptionEntryRepository> Handler<MarkSubscriptionEntryAsReadCommand>
    for MarkSubscriptionEntryAsReadHandler<SER>
{
    type Response = ();
    type Error = MarkSubscriptionEntryAsReadError;

    async fn handle(
        &self,
        cmd: MarkSubscriptionEntryAsReadCommand,
    ) -> Result<Self::Response, Self::Error> {
        let subscription_entry = self
            .subscription_entry_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| MarkSubscriptionEntryAsReadError::NotFound(cmd.id))?;

        self.subscription_entry_repository
            .mark_as_read(cmd.id)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkSubscriptionEntryAsReadError {
    #[error("subscription entry not found with ID: {0}")]
    NotFound(SubscriptionEntryId),

    #[error(transparent)]
    Core(#[from] SubscriptionEntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
