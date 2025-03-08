use uuid::Uuid;

use super::{ApiKey, ApiKeySearched, Cursor, Error};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn find_api_keys(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error>;

    async fn find_api_key_by_id(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyFindByIdParams,
    ) -> Result<ApiKeyById, Error>;

    async fn create_api_key(&self, params: ApiKeyCreateParams) -> Result<(), Error>;

    async fn update_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyUpdateParams,
    ) -> Result<(), Error>;

    async fn delete_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyDeleteParams,
    ) -> Result<(), Error>;

    async fn search_api_key(
        &self,
        params: ApiKeySearchParams,
    ) -> Result<Option<ApiKeySearched>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindByIdParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyCreateParams {
    pub id: Uuid,
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyDeleteParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeySearchParams {
    pub lookup_hash: String,
}
