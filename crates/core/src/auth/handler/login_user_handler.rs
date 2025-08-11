use chrono::Utc;
use colette_util::argon2_verify;
use jsonwebtoken::Header;

use crate::{
    Handler, RepositoryError,
    account::AccountRepository,
    auth::{AuthConfig, Claims, LOCAL_PROVIDER, TokenData, TokenType},
    user::{User, UserRepository},
};

#[derive(Debug, Clone)]
pub struct LoginUserCommand {
    pub email: String,
    pub password: String,
}

pub struct LoginUserHandler {
    account_repository: Box<dyn AccountRepository>,
    user_repository: Box<dyn UserRepository>,
    auth_config: AuthConfig,
}

impl LoginUserHandler {
    pub fn new(
        account_repository: impl AccountRepository,
        user_repository: impl UserRepository,
        auth_config: AuthConfig,
    ) -> Self {
        Self {
            account_repository: Box::new(account_repository),
            user_repository: Box::new(user_repository),
            auth_config,
        }
    }
}

#[async_trait::async_trait]
impl Handler<LoginUserCommand> for LoginUserHandler {
    type Response = TokenData;
    type Error = LoginUserError;

    async fn handle(&self, cmd: LoginUserCommand) -> Result<Self::Response, Self::Error> {
        let account = self
            .account_repository
            .find_by_sub_and_provider(cmd.email, LOCAL_PROVIDER.into())
            .await?
            .ok_or_else(|| LoginUserError::NotAuthenticated)?;

        let password_hash = account
            .password_hash
            .ok_or_else(|| LoginUserError::NotAuthenticated)?;

        let valid = argon2_verify(&cmd.password, &password_hash)?;
        if !valid {
            return Err(LoginUserError::NotAuthenticated);
        };

        let user = self
            .user_repository
            .find_by_id(account.user_id)
            .await?
            .ok_or_else(|| LoginUserError::NotAuthenticated)?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }
}

impl LoginUserHandler {
    fn generate_tokens(&self, user: User) -> Result<TokenData, LoginUserError> {
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
pub enum LoginUserError {
    #[error("not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Crypto(#[from] colette_util::CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
