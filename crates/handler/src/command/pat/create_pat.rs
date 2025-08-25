use chrono::{DateTime, Utc};
use colette_authentication::{
    LookupHash, PatError, PatId, PatPreview, PatRepository, PatTitle, PatValue,
    PersonalAccessToken, UserId, VerificationHash,
};
use colette_common::RepositoryError;
use colette_util::{
    CryptoError, argon2_hash, base64_encode, hex_encode, random_generate, sha256_hash,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct CreatePatCommand {
    pub title: String,
    pub user_id: UserId,
}

pub struct CreatePatHandler<PR: PatRepository> {
    pat_repository: PR,
}

impl<PR: PatRepository> CreatePatHandler<PR> {
    pub fn new(pat_repository: PR) -> Self {
        Self { pat_repository }
    }
}

impl<PR: PatRepository> Handler<CreatePatCommand> for CreatePatHandler<PR> {
    type Response = PatCreated;
    type Error = CreatePatError;

    async fn handle(&self, cmd: CreatePatCommand) -> Result<Self::Response, Self::Error> {
        let value = PatValue::new(base64_encode(&random_generate(32)))?;

        let lookup_hash = LookupHash::new(hex_encode(&sha256_hash(value.as_inner())))?;
        let verification_hash = VerificationHash::new(argon2_hash(value.as_inner())?)?;

        let title = PatTitle::new(cmd.title)?;
        let preview = PatPreview::new(&value);

        let pat =
            PersonalAccessToken::new(lookup_hash, verification_hash, title, preview, cmd.user_id);
        self.pat_repository.save(&pat).await?;

        Ok(PatCreated {
            id: pat.id(),
            title: pat.title().to_owned(),
            value,
            created_at: pat.created_at(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PatCreated {
    id: PatId,
    title: PatTitle,
    value: PatValue,
    created_at: DateTime<Utc>,
}

impl PatCreated {
    pub fn id(&self) -> PatId {
        self.id
    }

    pub fn title(&self) -> &PatTitle {
        &self.title
    }

    pub fn value(&self) -> &PatValue {
        &self.value
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePatError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Pat(#[from] PatError),

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
