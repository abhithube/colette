use uuid::Uuid;

use super::{Cursor, Error, FeedEntryFilter, Stream};
use crate::common::IdParams;

#[async_trait::async_trait]
pub trait StreamRepository: Send + Sync + 'static {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error>;

    async fn create_stream(&self, data: StreamCreateData) -> Result<Uuid, Error>;

    async fn update_stream(&self, params: IdParams, data: StreamUpdateData) -> Result<(), Error>;

    async fn delete_stream(&self, params: IdParams) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct StreamCreateData {
    pub title: String,
    pub filter: FeedEntryFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdateData {
    pub title: Option<String>,
    pub filter: Option<FeedEntryFilter>,
}
