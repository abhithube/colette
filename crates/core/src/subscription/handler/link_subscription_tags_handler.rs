use crate::{
    Handler,
    common::RepositoryError,
    subscription::{
        SubscriptionError, SubscriptionId, SubscriptionLinkTagParams, SubscriptionRepository,
    },
    tag::TagId,
    auth::UserId,
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
    type Response = ();
    type Error = LinkSubscriptionTagsError;

    async fn handle(
        &self,
        cmd: LinkSubscriptionTagsCommand,
    ) -> Result<Self::Response, Self::Error> {
        let subscription = self
            .subscription_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| LinkSubscriptionTagsError::NotFound(cmd.id))?;
        subscription.authorize(cmd.user_id)?;

        self.subscription_repository
            .link_tags(SubscriptionLinkTagParams {
                subscription_id: cmd.id,
                tag_ids: cmd.tag_ids,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkSubscriptionTagsError {
    #[error("subscription not found with ID: {0}")]
    NotFound(SubscriptionId),

    #[error(transparent)]
    Core(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
