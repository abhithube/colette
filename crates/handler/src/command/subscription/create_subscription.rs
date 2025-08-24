use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_core::{
    Subscription,
    feed::FeedId,
    subscription::{
        SubscriptionDescription, SubscriptionError, SubscriptionRepository, SubscriptionTitle,
    },
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct CreateSubscriptionCommand {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: FeedId,
    pub user_id: UserId,
}

pub struct CreateSubscriptionHandler<SR: SubscriptionRepository> {
    subscription_repository: SR,
}

impl<SR: SubscriptionRepository> CreateSubscriptionHandler<SR> {
    pub fn new(subscription_repository: SR) -> Self {
        Self {
            subscription_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SR: SubscriptionRepository> Handler<CreateSubscriptionCommand>
    for CreateSubscriptionHandler<SR>
{
    type Response = Subscription;
    type Error = CreateSubscriptionError;

    async fn handle(&self, cmd: CreateSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        let title = SubscriptionTitle::new(cmd.title)?;
        let description = cmd
            .description
            .map(SubscriptionDescription::new)
            .transpose()?;

        let subscription = Subscription::new(title, description, cmd.feed_id, cmd.user_id);

        self.subscription_repository
            .save(&subscription)
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => CreateSubscriptionError::Subscription(
                    SubscriptionError::Conflict(cmd.feed_id.as_inner()),
                ),
                _ => CreateSubscriptionError::Repository(e),
            })?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSubscriptionError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
