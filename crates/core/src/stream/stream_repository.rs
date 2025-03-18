use uuid::Uuid;

use super::{Error, Stream};

#[async_trait::async_trait]
pub trait StreamRepository: Send + Sync + 'static {
    async fn find(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Stream>, Error>;

    async fn save(&self, data: &Stream, upsert: bool) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
