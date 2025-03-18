use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{ApiKey, Error};

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error>;

    async fn find_one(&self, key: ApiKeyFindOne) -> Result<Option<ApiKey>, Error>;

    async fn save(&self, data: &ApiKey, upsert: bool) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<String>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ApiKeyFindOne {
    Id(Uuid),
    LookupHash(String),
}
