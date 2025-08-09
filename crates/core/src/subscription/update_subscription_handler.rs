use uuid::Uuid;

use super::{SubscriptionRepository, SubscriptionUpdateParams};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone, Default)]
pub struct UpdateSubscriptionCommand {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub user_id: Uuid,
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
        let Some(subscription) = self.subscription_repository.find_by_id(cmd.id).await? else {
            return Err(UpdateSubscriptionError::NotFound(cmd.id));
        };
        if subscription.user_id != cmd.user_id {
            return Err(UpdateSubscriptionError::Forbidden(cmd.id));
        }

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
    NotFound(Uuid),

    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
