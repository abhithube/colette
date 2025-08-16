use chrono::{DateTime, Utc};
use colette_util::{
    CryptoError, argon2_hash, base64_encode, hex_encode, random_generate, sha256_hash,
};

use crate::{
    Handler,
    auth::{
        LookupHash, PatId, PatPreview, PatTitle, PatValue, PersonalAccessToken, UserError, UserId,
        UserRepository, VerificationHash,
    },
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct CreatePatCommand {
    pub title: String,
    pub user_id: UserId,
}

pub struct CreatePatHandler {
    user_repository: Box<dyn UserRepository>,
}

impl CreatePatHandler {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreatePatCommand> for CreatePatHandler {
    type Response = PatCreated;
    type Error = CreatePatError;

    async fn handle(&self, cmd: CreatePatCommand) -> Result<Self::Response, Self::Error> {
        let mut user = self
            .user_repository
            .find_by_id(cmd.user_id)
            .await?
            .ok_or_else(|| CreatePatError::NotAuthenticated)?;

        let value = PatValue::new(base64_encode(&random_generate(32))).map_err(UserError::Pat)?;

        let lookup_hash =
            LookupHash::new(hex_encode(&sha256_hash(value.as_inner()))).map_err(UserError::Pat)?;
        let verification_hash =
            VerificationHash::new(argon2_hash(value.as_inner())?).map_err(UserError::Pat)?;

        let title = PatTitle::new(cmd.title).map_err(UserError::Pat)?;
        let preview = PatPreview::new(&value);

        let pat = PersonalAccessToken::new(lookup_hash, verification_hash, title, preview);
        let data = pat.clone();

        user.add_personal_access_token(pat);

        self.user_repository.save(&user).await?;

        Ok(PatCreated {
            id: data.id(),
            title: data.title().to_owned(),
            value,
            created_at: data.created_at(),
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
    User(#[from] UserError),

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
