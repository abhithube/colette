use uuid::Uuid;

use super::{Error, Stream};

#[async_trait::async_trait]
pub trait StreamRepository: Send + Sync + 'static {
    async fn query(&self, params: StreamParams) -> Result<Vec<Stream>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Stream>, Error>;

    async fn save(&self, data: &Stream) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamParams {
    pub id: Option<Uuid>,
    pub user_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
