use uuid::Uuid;

use super::{SubscriptionLinkTagParams, SubscriptionRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct LinkSubscriptionTagsCommand {
    pub id: Uuid,
    pub tag_ids: Vec<Uuid>,
    pub user_id: Uuid,
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
        let Some(subscription) = self.subscription_repository.find_by_id(cmd.id).await? else {
            return Err(LinkSubscriptionTagsError::NotFound(cmd.id));
        };
        if subscription.user_id != cmd.user_id {
            return Err(LinkSubscriptionTagsError::Forbidden(cmd.id));
        }

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
    NotFound(Uuid),

    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
