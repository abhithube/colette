use colette_authentication::UserRepository;
use colette_common::RepositoryError;
use colette_jwt::{Claims, JwtManager};
use uuid::Uuid;

use crate::{Handler, JwtConfig, TokenData, TokenType};

#[derive(Debug, Clone)]
pub struct RefreshAccessTokenCommand {
    pub refresh_token: String,
}

pub struct RefreshAccessTokenHandler<UR: UserRepository, JM: JwtManager> {
    user_repository: UR,
    jwt_manager: JM,
    jwt_config: JwtConfig,
}

impl<UR: UserRepository, JM: JwtManager> RefreshAccessTokenHandler<UR, JM> {
    pub fn new(user_repository: UR, jwt_manager: JM, jwt_config: JwtConfig) -> Self {
        Self {
            user_repository,
            jwt_manager,
            jwt_config,
        }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository, JM: JwtManager> Handler<RefreshAccessTokenCommand>
    for RefreshAccessTokenHandler<UR, JM>
{
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
            user.id().as_inner().to_string(),
            self.jwt_config.access_duration,
        ))?;
        let refresh_token = self.jwt_manager.generate(Claims::new(
            user.id().as_inner().to_string(),
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
