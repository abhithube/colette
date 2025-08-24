use colette_common::RepositoryError;
use colette_crud::SubscriptionError;
use uuid::Uuid;

use crate::{Handler, SubscriptionDto, SubscriptionQueryRepository};

#[derive(Debug, Clone)]
pub struct GetSubscriptionQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetSubscriptionHandler<SQR: SubscriptionQueryRepository> {
    subscription_query_repository: SQR,
}

impl<SQR: SubscriptionQueryRepository> GetSubscriptionHandler<SQR> {
    pub fn new(subscription_query_repository: SQR) -> Self {
        Self {
            subscription_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SQR: SubscriptionQueryRepository> Handler<GetSubscriptionQuery>
    for GetSubscriptionHandler<SQR>
{
    type Response = SubscriptionDto;
    type Error = GetSubscriptionError;

    async fn handle(&self, query: GetSubscriptionQuery) -> Result<Self::Response, Self::Error> {
        let subscription = self
            .subscription_query_repository
            .query_by_id(query.id, query.user_id)
            .await?
            .ok_or(SubscriptionError::NotFound(query.id))?;

        Ok(subscription)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetSubscriptionError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
