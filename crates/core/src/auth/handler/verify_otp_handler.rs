use colette_jwt::{Claims, JwtManager};

use crate::{
    Handler,
    auth::{JwtConfig, TokenData, TokenType, UserError, UserRepository},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct VerifyOtpCommand {
    pub email: String,
    pub code: String,
}

pub struct VerifyOtpHandler {
    user_repository: Box<dyn UserRepository>,
    jwt_manager: Box<dyn JwtManager>,
    jwt_config: JwtConfig,
}

impl VerifyOtpHandler {
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
impl Handler<VerifyOtpCommand> for VerifyOtpHandler {
    type Response = TokenData;
    type Error = LoginUserError;

    async fn handle(&self, cmd: VerifyOtpCommand) -> Result<Self::Response, Self::Error> {
        let mut user = self
            .user_repository
            .find_by_email(cmd.email.parse().map_err(UserError::InvalidEmail)?)
            .await?
            .ok_or_else(|| LoginUserError::NotAuthenticated)?;

        user.verify_otp_code(cmd.code)?;

        self.user_repository.save(&user).await?;

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
pub enum LoginUserError {
    #[error("not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Core(#[from] UserError),

    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
