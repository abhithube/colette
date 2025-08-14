use crate::{
    Handler,
    api_key::{ApiKey, ApiKeyError, ApiKeyId, ApiKeyRepository},
    common::RepositoryError,
    auth::UserId,
};

#[derive(Debug, Clone)]
pub struct GetApiKeyQuery {
    pub id: ApiKeyId,
    pub user_id: UserId,
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
        let api_key = self
            .api_key_repository
            .find_by_id(query.id)
            .await?
            .ok_or_else(|| GetApiKeyError::NotFound(query.id))?;
        api_key.authorize(query.user_id)?;

        Ok(api_key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetApiKeyError {
    #[error("API key not found with ID: {0}")]
    NotFound(ApiKeyId),

    #[error(transparent)]
    Core(#[from] ApiKeyError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
