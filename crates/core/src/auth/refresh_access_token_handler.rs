use chrono::Utc;
use jsonwebtoken::{Header, Validation};

use super::{AuthConfig, Claims, TokenData, TokenType};
use crate::{Handler, RepositoryError, User, user::UserRepository};

#[derive(Debug, Clone)]
pub struct RefreshAccessTokenCommand {
    pub refresh_token: String,
}

pub struct RefreshAccessTokenHandler {
    user_repository: Box<dyn UserRepository>,
    auth_config: AuthConfig,
}

impl RefreshAccessTokenHandler {
    pub fn new(user_repository: impl UserRepository, auth_config: AuthConfig) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            auth_config,
        }
    }
}

#[async_trait::async_trait]
impl Handler<RefreshAccessTokenCommand> for RefreshAccessTokenHandler {
    type Response = TokenData;
    type Error = RefreshAccessTokenError;

    async fn handle(&self, data: RefreshAccessTokenCommand) -> Result<Self::Response, Self::Error> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.auth_config.jwt.issuer]);
        validation.set_audience(&self.auth_config.jwt.audience);

        let token_data = jsonwebtoken::decode::<Claims>(
            &data.refresh_token,
            &self.auth_config.jwt.decoding_key,
            &validation,
        )?;

        let user = match self
            .user_repository
            .find_by_id(token_data.claims.sub)
            .await?
        {
            Some(user) => Ok(user),
            None => Err(RefreshAccessTokenError::NotAuthenticated),
        }?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }
}

impl RefreshAccessTokenHandler {
    fn generate_tokens(&self, user: User) -> Result<TokenData, RefreshAccessTokenError> {
        let now = Utc::now();

        let access_token = {
            let access_claims = Claims {
                iss: self.auth_config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.auth_config.jwt.audience.clone(),
                exp: (now + self.auth_config.jwt.access_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &access_claims,
                &self.auth_config.jwt.encoding_key,
            )?
        };

        let refresh_token = {
            let refresh_claims = Claims {
                iss: self.auth_config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.auth_config.jwt.audience.clone(),
                exp: (now + self.auth_config.jwt.refresh_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &refresh_claims,
                &self.auth_config.jwt.encoding_key,
            )?
        };

        Ok(TokenData {
            access_token,
            access_expires_in: self.auth_config.jwt.access_duration,
            refresh_token,
            refresh_expires_in: self.auth_config.jwt.refresh_duration,
            token_type: TokenType::default(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshAccessTokenError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
