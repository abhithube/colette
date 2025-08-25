use colette_authentication::{CodeValue, UserError, UserRepository};
use colette_common::RepositoryError;
use colette_jwt::{Claims, JwtManager};

use crate::{Handler, JwtConfig, TokenData, TokenType};

#[derive(Debug, Clone)]
pub struct VerifyOtpCommand {
    pub email: String,
    pub code: String,
}

pub struct VerifyOtpHandler<UR: UserRepository, JM: JwtManager> {
    user_repository: UR,
    jwt_manager: JM,
    jwt_config: JwtConfig,
}

impl<UR: UserRepository, JM: JwtManager> VerifyOtpHandler<UR, JM> {
    pub fn new(user_repository: UR, jwt_manager: JM, jwt_config: JwtConfig) -> Self {
        Self {
            user_repository,
            jwt_manager,
            jwt_config,
        }
    }
}

impl<UR: UserRepository, JM: JwtManager> Handler<VerifyOtpCommand> for VerifyOtpHandler<UR, JM> {
    type Response = TokenData;
    type Error = LoginUserError;

    async fn handle(&self, cmd: VerifyOtpCommand) -> Result<Self::Response, Self::Error> {
        let mut user = self
            .user_repository
            .find_by_email(cmd.email.parse().map_err(UserError::InvalidEmail)?)
            .await?
            .ok_or(LoginUserError::NotAuthenticated)?;

        let code_value = CodeValue::new(cmd.code).map_err(UserError::Otp)?;
        user.use_otp_code(code_value)?;

        self.user_repository.save(&user).await?;

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
pub enum LoginUserError {
    #[error("not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
