use uuid::Uuid;

use super::SubscriptionRepository;
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct DeleteSubscriptionCommand {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct DeleteSubscriptionHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl DeleteSubscriptionHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeleteSubscriptionCommand> for DeleteSubscriptionHandler {
    type Response = ();
    type Error = DeleteSubscriptionError;

    async fn handle(&self, cmd: DeleteSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        let Some(subscription) = self.subscription_repository.find_by_id(cmd.id).await? else {
            return Err(DeleteSubscriptionError::NotFound(cmd.id));
        };
        if subscription.user_id != cmd.user_id {
            return Err(DeleteSubscriptionError::Forbidden(cmd.id));
        }

        self.subscription_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSubscriptionError {
    #[error("subscription not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
