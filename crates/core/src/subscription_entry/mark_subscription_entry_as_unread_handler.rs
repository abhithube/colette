use uuid::Uuid;

use super::SubscriptionEntryRepository;
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct MarkSubscriptionEntryAsUnreadCommand {
    pub id: Uuid,
    pub user_id: Uuid,
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
        let Some(subscription_entry) = self
            .subscription_entry_repository
            .find_by_id(cmd.id)
            .await?
        else {
            return Err(MarkSubscriptionEntryAsUnreadError::NotFound(cmd.id));
        };
        if subscription_entry.user_id != cmd.user_id {
            return Err(MarkSubscriptionEntryAsUnreadError::Forbidden(cmd.id));
        }

        self.subscription_entry_repository
            .mark_as_unread(cmd.id)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkSubscriptionEntryAsUnreadError {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
