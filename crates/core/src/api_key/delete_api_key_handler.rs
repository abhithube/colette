use uuid::Uuid;

use super::ApiKeyRepository;
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct DeleteApiKeyCommand {
    pub id: Uuid,
    pub user_id: Uuid,
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
        let Some(api_key) = self.api_key_repository.find_by_id(cmd.id).await? else {
            return Err(DeleteApiKeyError::NotFound(cmd.id));
        };
        if api_key.user_id != cmd.user_id {
            return Err(DeleteApiKeyError::Forbidden(cmd.id));
        }

        self.api_key_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteApiKeyError {
    #[error("API key not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
