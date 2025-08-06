use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{ApiKey, Error};

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKeyById>, Error>;

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: String,
    ) -> Result<Option<ApiKeyByLookupHash>, Error>;

    async fn insert(&self, params: ApiKeyInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: ApiKeyUpdateParams) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ApiKeyByLookupHash {
    pub id: Uuid,
    pub verification_hash: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ApiKeyInsertParams {
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ApiKeyUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
}
