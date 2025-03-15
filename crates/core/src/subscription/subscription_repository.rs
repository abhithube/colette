use uuid::Uuid;

use super::{Error, Subscription};

#[async_trait::async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn find(&self, params: SubscriptionFindParams) -> Result<Vec<Subscription>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscription>, Error>;

    async fn save(&self, data: &Subscription, upsert: bool) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionFindParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<u64>,
}
