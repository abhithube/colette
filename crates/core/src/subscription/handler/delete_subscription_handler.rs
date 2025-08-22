use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    subscription::{SubscriptionError, SubscriptionId, SubscriptionRepository},
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
        self.subscription_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeleteSubscriptionError::Subscription(SubscriptionError::NotFound(cmd.id))
                }
                _ => DeleteSubscriptionError::Repository(e),
            })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSubscriptionError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
