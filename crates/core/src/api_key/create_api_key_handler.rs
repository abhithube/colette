use chrono::{DateTime, Utc};
use colette_util::{
    CryptoError, argon2_hash, base64_encode, hex_encode, random_generate, sha256_hash,
};
use uuid::Uuid;

use super::ApiKeyRepository;
use crate::{Handler, RepositoryError, api_key::ApiKeyInsertParams};

#[derive(Debug, Clone)]
pub struct CreateApiKeyCommand {
    pub title: String,
    pub user_id: Uuid,
}

pub struct CreateApiKeyHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl CreateApiKeyHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreateApiKeyCommand> for CreateApiKeyHandler {
    type Response = ApiKeyCreated;
    type Error = CreateApiKeyError;

    async fn handle(&self, cmd: CreateApiKeyCommand) -> Result<Self::Response, Self::Error> {
        let value = base64_encode(&random_generate(32));

        let lookup_hash = hex_encode(&sha256_hash(&value));
        let verification_hash = argon2_hash(&value)?;

        let preview = format!(
            "{}...{}",
            &value[0..8],
            &value[value.len() - 4..value.len()]
        );

        let api_key = self
            .api_key_repository
            .insert(ApiKeyInsertParams {
                lookup_hash,
                verification_hash,
                title: cmd.title,
                preview,
                user_id: cmd.user_id,
            })
            .await?;

        Ok(ApiKeyCreated {
            id: api_key.id,
            title: api_key.title,
            value,
            created_at: api_key.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyCreated {
    pub id: Uuid,
    pub title: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateApiKeyError {
    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
