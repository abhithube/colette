use colette_opml::Outline;
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

    async fn import(&self, data: ImportSubscriptionsData) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<u64>,
    pub with_feed: bool,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

pub struct ImportSubscriptionsData {
    pub outlines: Vec<Outline>,
    pub user_id: Uuid,
}
