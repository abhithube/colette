use uuid::Uuid;

use super::{Subscription, SubscriptionFindParams, SubscriptionRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetSubscriptionQuery {
    pub id: Uuid,
    pub with_unread_count: bool,
    pub with_tags: bool,
    pub user_id: Uuid,
}

pub struct GetSubscriptionHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl GetSubscriptionHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetSubscriptionQuery> for GetSubscriptionHandler {
    type Response = Subscription;
    type Error = GetSubscriptionError;

    async fn handle(&self, query: GetSubscriptionQuery) -> Result<Self::Response, Self::Error> {
        let mut subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                id: Some(query.id),
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;
        if subscriptions.is_empty() {
            return Err(GetSubscriptionError::NotFound(query.id));
        }

        let subscription = subscriptions.swap_remove(0);
        if subscription.user_id != query.user_id {
            return Err(GetSubscriptionError::Forbidden(query.id));
        }

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionError {
    #[error("subscription not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
