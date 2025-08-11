use crate::{
    Handler, RepositoryError,
    subscription::{
        Subscription, SubscriptionError, SubscriptionFindParams, SubscriptionId,
        SubscriptionRepository,
    },
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct GetSubscriptionQuery {
    pub id: SubscriptionId,
    pub with_unread_count: bool,
    pub with_tags: bool,
    pub user_id: UserId,
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
        subscription.authorize(query.user_id)?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionError {
    #[error("subscription not found with ID: {0}")]
    NotFound(SubscriptionId),

    #[error(transparent)]
    Core(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
