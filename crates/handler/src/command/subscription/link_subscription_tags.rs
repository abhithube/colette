use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{
    Subscription, SubscriptionError, SubscriptionId, SubscriptionRepository, TagId,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct LinkSubscriptionTagsCommand {
    pub id: SubscriptionId,
    pub tag_ids: Vec<TagId>,
    pub user_id: UserId,
}

pub struct LinkSubscriptionTagsHandler<SR: SubscriptionRepository> {
    subscription_repository: SR,
}

impl<SR: SubscriptionRepository> LinkSubscriptionTagsHandler<SR> {
    pub fn new(subscription_repository: SR) -> Self {
        Self {
            subscription_repository,
        }
    }
}

impl<SR: SubscriptionRepository> Handler<LinkSubscriptionTagsCommand>
    for LinkSubscriptionTagsHandler<SR>
{
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
            .ok_or(SubscriptionError::NotFound(cmd.id.as_inner()))?;

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
