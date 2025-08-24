use colette_core::{
    auth::UserId,
    common::RepositoryError,
    subscription::{
        SubscriptionDto, SubscriptionError, SubscriptionFindParams, SubscriptionId,
        SubscriptionRepository,
    },
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct GetSubscriptionQuery {
    pub id: SubscriptionId,
    pub user_id: UserId,
}

pub struct GetSubscriptionHandler<SR: SubscriptionRepository> {
    subscription_repository: SR,
}

impl<SR: SubscriptionRepository> GetSubscriptionHandler<SR> {
    pub fn new(subscription_repository: SR) -> Self {
        Self {
            subscription_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SR: SubscriptionRepository> Handler<GetSubscriptionQuery> for GetSubscriptionHandler<SR> {
    type Response = SubscriptionDto;
    type Error = GetSubscriptionError;

    async fn handle(&self, query: GetSubscriptionQuery) -> Result<Self::Response, Self::Error> {
        let mut subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: query.user_id,
                id: Some(query.id),
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;
        if subscriptions.is_empty() {
            return Err(GetSubscriptionError::Subscription(
                SubscriptionError::NotFound(query.id),
            ));
        }

        Ok(subscriptions.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
