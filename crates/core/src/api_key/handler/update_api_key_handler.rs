use crate::{
    Handler, RepositoryError,
    api_key::{ApiKeyError, ApiKeyId, ApiKeyRepository, ApiKeyUpdateParams},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct UpdateApiKeyCommand {
    pub id: ApiKeyId,
    pub title: Option<String>,
    pub user_id: UserId,
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
        let api_key = self
            .api_key_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateApiKeyError::NotFound(cmd.id))?;
        api_key.authorize(cmd.user_id)?;

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
    NotFound(ApiKeyId),

    #[error(transparent)]
    Core(#[from] ApiKeyError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
