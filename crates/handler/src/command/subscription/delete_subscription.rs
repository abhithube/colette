use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_core::subscription::{SubscriptionError, SubscriptionId, SubscriptionRepository};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeleteSubscriptionCommand {
    pub id: SubscriptionId,
    pub user_id: UserId,
}

pub struct DeleteSubscriptionHandler<SR: SubscriptionRepository> {
    subscription_repository: SR,
}

impl<SR: SubscriptionRepository> DeleteSubscriptionHandler<SR> {
    pub fn new(subscription_repository: SR) -> Self {
        Self {
            subscription_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SR: SubscriptionRepository> Handler<DeleteSubscriptionCommand>
    for DeleteSubscriptionHandler<SR>
{
    type Response = ();
    type Error = DeleteSubscriptionError;

    async fn handle(&self, cmd: DeleteSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        self.subscription_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => DeleteSubscriptionError::Subscription(
                    SubscriptionError::NotFound(cmd.id.as_inner()),
                ),
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
