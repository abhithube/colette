use uuid::Uuid;

use super::{Cursor, Error, Stream, SubscriptionEntryFilter};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait StreamRepository: Send + Sync + 'static {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error>;

    async fn find_stream_by_id(&self, tx: &dyn Transaction, id: Uuid) -> Result<StreamById, Error>;

    async fn create_stream(&self, data: StreamCreateData) -> Result<Uuid, Error>;

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: StreamUpdateData,
    ) -> Result<(), Error>;

    async fn delete_stream(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct StreamCreateData {
    pub title: String,
    pub filter: SubscriptionEntryFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdateData {
    pub title: Option<String>,
    pub filter: Option<SubscriptionEntryFilter>,
}
