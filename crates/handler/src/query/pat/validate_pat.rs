use colette_core::{
    auth::UserId,
    common::RepositoryError,
    pat::{LookupHash, PatError, PatRepository},
};
use colette_util::{CryptoError, argon2_verify, hex_encode, sha256_hash};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ValidatePatQuery {
    pub value: String,
}

pub struct ValidatePatHandler<PR: PatRepository> {
    pat_repository: PR,
}

impl<PR: PatRepository> ValidatePatHandler<PR> {
    pub fn new(pat_repository: PR) -> Self {
        Self { pat_repository }
    }
}

#[async_trait::async_trait]
impl<PR: PatRepository> Handler<ValidatePatQuery> for ValidatePatHandler<PR> {
    type Response = UserId;
    type Error = ValidatePatError;

    async fn handle(&self, cmd: ValidatePatQuery) -> Result<Self::Response, Self::Error> {
        let lookup_hash = LookupHash::new(hex_encode(&sha256_hash(&cmd.value)))?;

        let pat = self
            .pat_repository
            .find_by_lookup_hash(&lookup_hash)
            .await?
            .ok_or(ValidatePatError::InvalidPat)?;

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
