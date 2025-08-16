use colette_util::{CryptoError, argon2_verify, hex_encode, sha256_hash};

use crate::{
    Handler,
    auth::{LookupHash, PatError, PatRepository, UserId},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct ValidatePatQuery {
    pub value: String,
}

pub struct ValidatePatHandler {
    pat_repository: Box<dyn PatRepository>,
}

impl ValidatePatHandler {
    pub fn new(pat_repository: impl PatRepository) -> Self {
        Self {
            pat_repository: Box::new(pat_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ValidatePatQuery> for ValidatePatHandler {
    type Response = UserId;
    type Error = ValidatePatError;

    async fn handle(&self, cmd: ValidatePatQuery) -> Result<Self::Response, Self::Error> {
        let lookup_hash = LookupHash::new(hex_encode(&sha256_hash(&cmd.value)))?;

        let pat = self
            .pat_repository
            .find_by_lookup_hash(&lookup_hash)
            .await?
            .ok_or_else(|| ValidatePatError::InvalidPat)?;

        let valid = argon2_verify(lookup_hash.as_inner(), pat.verification_hash().as_inner())?;
        if !valid {
            return Err(ValidatePatError::InvalidPat);
        }

        Ok(pat.user_id())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidatePatError {
    #[error("invalid PAT")]
    InvalidPat,

    #[error(transparent)]
    Pat(#[from] PatError),

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
