use crate::{
    Handler, Subscription,
    auth::UserId,
    common::RepositoryError,
    feed::FeedId,
    subscription::{
        SubscriptionDescription, SubscriptionError, SubscriptionRepository, SubscriptionTitle,
    },
};

#[derive(Debug, Clone)]
pub struct CreateSubscriptionCommand {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: FeedId,
    pub user_id: UserId,
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
                RepositoryError::Duplicate => CreateSubscriptionError::Conflict(cmd.feed_id),
                _ => CreateSubscriptionError::Repository(e),
            })?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSubscriptionError {
    #[error("already subscribed to feed with ID: {0}")]
    Conflict(FeedId),

    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
