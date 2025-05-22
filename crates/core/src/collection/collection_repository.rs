use uuid::Uuid;

use super::{Collection, Error};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn query(&self, params: CollectionParams) -> Result<Vec<Collection>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Collection>, Error> {
        Ok(self
            .query(CollectionParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &Collection) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
