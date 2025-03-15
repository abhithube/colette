use uuid::Uuid;

use super::{Collection, Error};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Collection>, Error>;

    async fn save(&self, data: &Collection, upsert: bool) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
