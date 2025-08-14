use chrono::{DateTime, Utc};

use crate::{ApiKey, api_key::ApiKeyId, auth::UserId, common::RepositoryError};

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync + 'static {
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, RepositoryError>;

    async fn find_by_id(&self, id: ApiKeyId) -> Result<Option<ApiKey>, RepositoryError> {
        let mut api_keys = self
            .find(ApiKeyFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if api_keys.is_empty() {
            return Ok(None);
        }

        Ok(Some(api_keys.swap_remove(0)))
    }

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: String,
    ) -> Result<Option<ApiKey>, RepositoryError> {
        let mut api_keys = self
            .find(ApiKeyFindParams {
                lookup_hash: Some(lookup_hash),
                ..Default::default()
            })
            .await?;
        if api_keys.is_empty() {
            return Ok(None);
        }

        Ok(Some(api_keys.swap_remove(0)))
    }

    async fn insert(&self, params: ApiKeyInsertParams) -> Result<ApiKey, RepositoryError>;

    async fn update(&self, params: ApiKeyUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: ApiKeyId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindParams {
    pub id: Option<ApiKeyId>,
    pub lookup_hash: Option<String>,
    pub user_id: Option<UserId>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyInsertParams {
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct ApiKeyUpdateParams {
    pub id: ApiKeyId,
    pub title: Option<String>,
}
