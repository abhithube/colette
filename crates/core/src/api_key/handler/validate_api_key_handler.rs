use colette_util::{CryptoError, argon2_verify, hex_encode, sha256_hash};

use crate::{ApiKey, Handler, api_key::ApiKeyRepository, common::RepositoryError};

#[derive(Debug, Clone)]
pub struct ValidateApiKeyQuery {
    pub value: String,
}

pub struct ValidateApiKeyHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl ValidateApiKeyHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ValidateApiKeyQuery> for ValidateApiKeyHandler {
    type Response = ApiKey;
    type Error = ValidateApiKeyError;

    async fn handle(&self, cmd: ValidateApiKeyQuery) -> Result<Self::Response, Self::Error> {
        let lookup_hash = hex_encode(&sha256_hash(&cmd.value));

        let api_key = self
            .api_key_repository
            .find_by_lookup_hash(lookup_hash)
            .await?
            .ok_or_else(|| ValidateApiKeyError::InvalidApiKey)?;

        let valid = argon2_verify(&cmd.value, &api_key.verification_hash)?;
        if !valid {
            return Err(ValidateApiKeyError::InvalidApiKey);
        }

        Ok(api_key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateApiKeyError {
    #[error("invalid API key")]
    InvalidApiKey,

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
