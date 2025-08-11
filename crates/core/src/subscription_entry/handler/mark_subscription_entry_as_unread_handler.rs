use crate::{
    Handler, RepositoryError,
    subscription_entry::{
        SubscriptionEntryError, SubscriptionEntryId, SubscriptionEntryRepository,
    },
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct MarkSubscriptionEntryAsUnreadCommand {
    pub id: SubscriptionEntryId,
    pub user_id: UserId,
}

pub struct MarkSubscriptionEntryAsUnreadHandler {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
}

impl MarkSubscriptionEntryAsUnreadHandler {
    pub fn new(subscription_entry_repository: impl SubscriptionEntryRepository) -> Self {
        Self {
            subscription_entry_repository: Box::new(subscription_entry_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<MarkSubscriptionEntryAsUnreadCommand> for MarkSubscriptionEntryAsUnreadHandler {
    type Response = ();
    type Error = MarkSubscriptionEntryAsUnreadError;

    async fn handle(
        &self,
        cmd: MarkSubscriptionEntryAsUnreadCommand,
    ) -> Result<Self::Response, Self::Error> {
        let subscription_entry = self
            .subscription_entry_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| MarkSubscriptionEntryAsUnreadError::NotFound(cmd.id))?;
        subscription_entry.authorize(cmd.user_id)?;

        self.subscription_entry_repository
            .mark_as_unread(cmd.id)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkSubscriptionEntryAsUnreadError {
    #[error("subscription entry not found with ID: {0}")]
    NotFound(SubscriptionEntryId),

    #[error(transparent)]
    Core(#[from] SubscriptionEntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
