use uuid::Uuid;

use super::ApiKeyRepository;
use crate::{Handler, RepositoryError, api_key::ApiKeyUpdateParams};

#[derive(Debug, Clone, Default)]
pub struct UpdateApiKeyCommand {
    pub id: Uuid,
    pub title: Option<String>,
    pub user_id: Uuid,
}

pub struct UpdateApiKeyHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl UpdateApiKeyHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<UpdateApiKeyCommand> for UpdateApiKeyHandler {
    type Response = ();
    type Error = UpdateApiKeyError;

    async fn handle(&self, cmd: UpdateApiKeyCommand) -> Result<Self::Response, Self::Error> {
        let Some(api_key) = self.api_key_repository.find_by_id(cmd.id).await? else {
            return Err(UpdateApiKeyError::NotFound(cmd.id));
        };
        if api_key.user_id != cmd.user_id {
            return Err(UpdateApiKeyError::Forbidden(cmd.id));
        }

        self.api_key_repository
            .update(ApiKeyUpdateParams {
                id: cmd.id,
                title: cmd.title,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateApiKeyError {
    #[error("API key not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
