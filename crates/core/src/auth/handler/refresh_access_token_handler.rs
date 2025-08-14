use colette_jwt::{Claims, JwtManager};
use uuid::Uuid;

use crate::{
    Handler,
    auth::{JwtConfig, TokenData, TokenType, UserRepository},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct RefreshAccessTokenCommand {
    pub refresh_token: String,
}

pub struct RefreshAccessTokenHandler {
    user_repository: Box<dyn UserRepository>,
    jwt_manager: Box<dyn JwtManager>,
    jwt_config: JwtConfig,
}

impl RefreshAccessTokenHandler {
    pub fn new(
        user_repository: impl UserRepository,
        jwt_manager: impl JwtManager,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            jwt_manager: Box::new(jwt_manager),
            jwt_config,
        }
    }
}

#[async_trait::async_trait]
impl Handler<RefreshAccessTokenCommand> for RefreshAccessTokenHandler {
    type Response = TokenData;
    type Error = RefreshAccessTokenError;

    async fn handle(&self, cmd: RefreshAccessTokenCommand) -> Result<Self::Response, Self::Error> {
        let claims = self.jwt_manager.verify(&cmd.refresh_token)?;

        let user = match self
            .user_repository
            .find_by_id(claims.sub().parse::<Uuid>().unwrap().into())
            .await?
        {
            Some(user) => Ok(user),
            None => Err(RefreshAccessTokenError::NotAuthenticated),
        }?;

        let access_token = self.jwt_manager.generate(Claims::new(
            user.id().to_string(),
            self.jwt_config.access_duration,
        ))?;
        let refresh_token = self.jwt_manager.generate(Claims::new(
            user.id().to_string(),
            self.jwt_config.refresh_duration,
        ))?;

        Ok(TokenData {
            access_token,
            access_expires_in: self.jwt_config.access_duration,
            refresh_token,
            refresh_expires_in: self.jwt_config.refresh_duration,
            token_type: TokenType::Bearer,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshAccessTokenError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
