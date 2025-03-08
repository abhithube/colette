use uuid::Uuid;

use super::{Cursor, Error, Stream, SubscriptionEntryFilter};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait StreamRepository: Send + Sync + 'static {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error>;

    async fn find_stream_by_id(
        &self,
        tx: &dyn Transaction,
        params: StreamFindByIdParams,
    ) -> Result<StreamById, Error>;

    async fn create_stream(&self, params: StreamCreateParams) -> Result<(), Error>;

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamUpdateParams,
    ) -> Result<(), Error>;

    async fn delete_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamDeleteParams,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindByIdParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct StreamCreateParams {
    pub id: Uuid,
    pub title: String,
    pub filter: SubscriptionEntryFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub filter: Option<SubscriptionEntryFilter>,
}

#[derive(Debug, Clone, Default)]
pub struct StreamDeleteParams {
    pub id: Uuid,
}
