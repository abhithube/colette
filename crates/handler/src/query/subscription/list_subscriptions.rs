use colette_common::RepositoryError;
use uuid::Uuid;

use crate::{
    Handler, Paginated, SubscriptionCursor, SubscriptionDto, SubscriptionQueryParams,
    SubscriptionQueryRepository, paginate,
};

#[derive(Debug, Clone)]
pub struct ListSubscriptionsQuery {
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<SubscriptionCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListSubscriptionsHandler<SQR: SubscriptionQueryRepository> {
    subscription_query_repository: SQR,
}

impl<SQR: SubscriptionQueryRepository> ListSubscriptionsHandler<SQR> {
    pub fn new(subscription_query_repository: SQR) -> Self {
        Self {
            subscription_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SQR: SubscriptionQueryRepository> Handler<ListSubscriptionsQuery>
    for ListSubscriptionsHandler<SQR>
{
    type Response = Paginated<SubscriptionDto, SubscriptionCursor>;
    type Error = ListSubscriptionsError;

    async fn handle(&self, query: ListSubscriptionsQuery) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .subscription_query_repository
            .query(SubscriptionQueryParams {
                user_id: query.user_id,
                tags: query.tags,
                cursor: query.cursor.map(|e| (e.title, e.id)),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(subscriptions, limit))
        } else {
            Ok(Paginated {
                items: subscriptions,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListSubscriptionsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
