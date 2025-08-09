use uuid::Uuid;

use super::{ApiKey, ApiKeyFindParams, ApiKeyRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetApiKeyQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetApiKeyHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl GetApiKeyHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetApiKeyQuery> for GetApiKeyHandler {
    type Response = ApiKey;
    type Error = GetApiKeyError;

    async fn handle(&self, query: GetApiKeyQuery) -> Result<Self::Response, Self::Error> {
        let mut api_keys = self
            .api_key_repository
            .find(ApiKeyFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if api_keys.is_empty() {
            return Err(GetApiKeyError::NotFound(query.id));
        }

        let api_key = api_keys.swap_remove(0);
        if api_key.user_id != query.user_id {
            return Err(GetApiKeyError::Forbidden(query.id));
        }

        Ok(api_key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetApiKeyError {
    #[error("API key not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
