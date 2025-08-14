use crate::{
    Handler,
    api_key::{ApiKeyError, ApiKeyId, ApiKeyRepository},
    common::RepositoryError,
    auth::UserId,
};

#[derive(Debug, Clone)]
pub struct DeleteApiKeyCommand {
    pub id: ApiKeyId,
    pub user_id: UserId,
}

pub struct DeleteApiKeyHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl DeleteApiKeyHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeleteApiKeyCommand> for DeleteApiKeyHandler {
    type Response = ();
    type Error = DeleteApiKeyError;

    async fn handle(&self, cmd: DeleteApiKeyCommand) -> Result<Self::Response, Self::Error> {
        let api_key = self
            .api_key_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| DeleteApiKeyError::NotFound(cmd.id))?;
        api_key.authorize(cmd.user_id)?;

        self.api_key_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteApiKeyError {
    #[error("API key not found with ID: {0}")]
    NotFound(ApiKeyId),

    #[error(transparent)]
    Core(#[from] ApiKeyError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
