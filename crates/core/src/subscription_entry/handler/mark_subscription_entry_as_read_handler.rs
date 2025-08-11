use crate::{
    Handler,
    common::RepositoryError,
    subscription_entry::{
        SubscriptionEntryError, SubscriptionEntryId, SubscriptionEntryRepository,
    },
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct MarkSubscriptionEntryAsReadCommand {
    pub id: SubscriptionEntryId,
    pub user_id: UserId,
}

pub struct MarkSubscriptionEntryAsReadHandler {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
}

impl MarkSubscriptionEntryAsReadHandler {
    pub fn new(subscription_entry_repository: impl SubscriptionEntryRepository) -> Self {
        Self {
            subscription_entry_repository: Box::new(subscription_entry_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<MarkSubscriptionEntryAsReadCommand> for MarkSubscriptionEntryAsReadHandler {
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
        subscription_entry.authorize(cmd.user_id)?;

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
