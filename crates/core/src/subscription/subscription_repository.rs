use uuid::Uuid;

use super::{Error, Subscription};

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscription>, Error> {
        Ok(self
            .query(SubscriptionParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<String>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<u64>,
    pub with_feeds: bool,
}
