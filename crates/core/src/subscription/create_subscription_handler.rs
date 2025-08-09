use uuid::Uuid;

use super::{SubscriptionInsertParams, SubscriptionRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct CreateSubscriptionCommand {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    pub user_id: Uuid,
}

pub struct CreateSubscriptionHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl CreateSubscriptionHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreateSubscriptionCommand> for CreateSubscriptionHandler {
    type Response = SubscriptionCreated;
    type Error = CreateSubscriptionError;

    async fn handle(&self, cmd: CreateSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .subscription_repository
            .insert(SubscriptionInsertParams {
                title: cmd.title,
                description: cmd.description,
                feed_id: cmd.feed_id,
                user_id: cmd.user_id,
            })
            .await?;

        Ok(SubscriptionCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreated {
    pub id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSubscriptionError {
    #[error("already subscribed to feed with ID: {0}")]
    Conflict(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
