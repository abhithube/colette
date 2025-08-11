use crate::{
    Handler,
    common::RepositoryError,
    subscription::{SubscriptionError, SubscriptionId, SubscriptionRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct DeleteSubscriptionCommand {
    pub id: SubscriptionId,
    pub user_id: UserId,
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
        let subscription = self
            .subscription_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| DeleteSubscriptionError::NotFound(cmd.id))?;
        subscription.authorize(cmd.user_id)?;

        self.subscription_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSubscriptionError {
    #[error("subscription not found with ID: {0}")]
    NotFound(SubscriptionId),

    #[error(transparent)]
    Core(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
