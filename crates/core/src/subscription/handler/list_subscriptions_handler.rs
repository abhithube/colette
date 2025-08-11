use crate::{
    Handler,
    common::RepositoryError,
    pagination::{Paginated, paginate},
    subscription::{
        Subscription, SubscriptionCursor, SubscriptionFindParams, SubscriptionRepository,
    },
    tag::TagId,
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct ListSubscriptionsQuery {
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<SubscriptionCursor>,
    pub limit: Option<usize>,
    pub with_unread_count: bool,
    pub with_tags: bool,
    pub user_id: UserId,
}

pub struct ListSubscriptionsHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl ListSubscriptionsHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListSubscriptionsQuery> for ListSubscriptionsHandler {
    type Response = Paginated<Subscription, SubscriptionCursor>;
    type Error = ListSubscriptionsError;

    async fn handle(&self, query: ListSubscriptionsQuery) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: Some(query.user_id),
                tags: query.tags,
                cursor: query.cursor.map(|e| (e.title, e.id)),
                limit: query.limit.map(|e| e + 1),
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
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
