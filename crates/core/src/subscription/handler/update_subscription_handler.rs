use crate::{
    Handler, Subscription,
    auth::UserId,
    common::RepositoryError,
    subscription::{
        SubscriptionDescription, SubscriptionError, SubscriptionId, SubscriptionRepository,
        SubscriptionTitle,
    },
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
    type Response = Subscription;
    type Error = UpdateSubscriptionError;

    async fn handle(&self, cmd: UpdateSubscriptionCommand) -> Result<Self::Response, Self::Error> {
        let mut subscription = self
            .subscription_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| {
                UpdateSubscriptionError::Subscription(SubscriptionError::NotFound(cmd.id))
            })?;

        if let Some(title) = cmd.title.map(SubscriptionTitle::new).transpose()? {
            subscription.set_title(title);
        }
        if let Some(description) = cmd.description {
            if let Some(description) = description.map(SubscriptionDescription::new).transpose()? {
                subscription.set_description(description);
            } else {
                subscription.remove_description();
            }
        }

        self.subscription_repository.save(&subscription).await?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateSubscriptionError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
