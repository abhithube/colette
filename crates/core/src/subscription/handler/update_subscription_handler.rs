use crate::{
    Handler,
    common::RepositoryError,
    subscription::{
        SubscriptionError, SubscriptionId, SubscriptionRepository, SubscriptionUpdateParams,
    },
    auth::UserId,
};

#[derive(Debug, Clone)]
pub struct UpdateSubscriptionCommand {
    pub id: SubscriptionId,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub user_id: UserId,
}

pub struct UpdateSubscriptionHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl UpdateSubscriptionHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<UpdateSubscriptionCommand> for UpdateSubscriptionHandler {
    type Response = ();
    type Error = UpdateSubscriptionError;

    async fn handle(&self, cmd: UpdateSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        let subscription = self
            .subscription_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateSubscriptionError::NotFound(cmd.id))?;
        subscription.authorize(cmd.user_id)?;

        self.subscription_repository
            .update(SubscriptionUpdateParams {
                id: cmd.id,
                title: cmd.title,
                description: cmd.description,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateSubscriptionError {
    #[error("subscription not found with ID: {0}")]
    NotFound(SubscriptionId),

    #[error(transparent)]
    Core(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
