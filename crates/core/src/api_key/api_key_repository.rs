use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{ApiKey, Error};

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn query(&self, params: ApiKeyParams) -> Result<Vec<ApiKey>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>, Error> {
        Ok(self
            .query(ApiKeyParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn find_by_lookup_hash(&self, lookup_hash: String) -> Result<Option<ApiKey>, Error> {
        Ok(self
            .query(ApiKeyParams {
                lookup_hash: Some(lookup_hash),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &ApiKey) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyParams {
    pub id: Option<Uuid>,
    pub lookup_hash: Option<String>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
