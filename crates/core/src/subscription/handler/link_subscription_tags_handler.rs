use crate::{
    Handler, Subscription,
    auth::UserId,
    common::RepositoryError,
    subscription::{SubscriptionError, SubscriptionId, SubscriptionRepository},
    tag::TagId,
};

#[derive(Debug, Clone)]
pub struct LinkSubscriptionTagsCommand {
    pub id: SubscriptionId,
    pub tag_ids: Vec<TagId>,
    pub user_id: UserId,
}

pub struct LinkSubscriptionTagsHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl LinkSubscriptionTagsHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<LinkSubscriptionTagsCommand> for LinkSubscriptionTagsHandler {
    type Response = Subscription;
    type Error = LinkSubscriptionTagsError;

    async fn handle(
        &self,
        cmd: LinkSubscriptionTagsCommand,
    ) -> Result<Self::Response, Self::Error> {
        let mut subscription = self
            .subscription_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| {
                LinkSubscriptionTagsError::Subscription(SubscriptionError::NotFound(cmd.id))
            })?;

        subscription.set_tags(cmd.tag_ids)?;

        self.subscription_repository.save(&subscription).await?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkSubscriptionTagsError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
