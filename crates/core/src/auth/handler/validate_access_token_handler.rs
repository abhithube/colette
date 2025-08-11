use jsonwebtoken::Validation;

use crate::{
    Handler,
    auth::{AuthConfig, Claims},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct ValidateAccessTokenQuery {
    pub access_token: String,
}

pub struct ValidateAccessTokenHandler {
    auth_config: AuthConfig,
}

impl ValidateAccessTokenHandler {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self { auth_config }
    }
}

#[async_trait::async_trait]
impl Handler<ValidateAccessTokenQuery> for ValidateAccessTokenHandler {
    type Response = Claims;
    type Error = ValidateAccessTokenError;

    async fn handle(&self, query: ValidateAccessTokenQuery) -> Result<Self::Response, Self::Error> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.auth_config.jwt.issuer]);
        validation.set_audience(&self.auth_config.jwt.audience);

        let token_data = jsonwebtoken::decode::<Claims>(
            &query.access_token,
            &self.auth_config.jwt.decoding_key,
            &validation,
        )?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateAccessTokenError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
