use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{ApiKey, Error};

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn query(&self, params: ApiKeyParams) -> Result<Vec<ApiKey>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>, Error>;

    async fn find_by_lookup_hash(&self, lookup_hash: String) -> Result<Option<ApiKey>, Error>;

    async fn save(&self, data: &ApiKey) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyParams {
    pub id: Option<Uuid>,
    pub user_id: Option<String>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}
