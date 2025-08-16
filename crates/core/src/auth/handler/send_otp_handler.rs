use chrono::{DateTime, Utc};

use crate::{
    Handler, User,
    auth::{OtpCode, OtpError, UserError, UserRepository},
    common::RepositoryError,
};

const MAX_ATTEMPTS: i8 = 5;

#[derive(Debug, Clone)]
pub struct SendOtpCommand {
    pub email: String,
}

pub struct SendOtpHandler {
    user_repository: Box<dyn UserRepository>,
}

impl SendOtpHandler {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<SendOtpCommand> for SendOtpHandler {
    type Response = OtpData;
    type Error = SendOtpError;

    async fn handle(&self, cmd: SendOtpCommand) -> Result<Self::Response, Self::Error> {
        let mut user = match self
            .user_repository
            .find_by_email(cmd.email.parse().map_err(UserError::InvalidEmail)?)
            .await?
        {
            Some(user) => user,
            None => User::new(cmd.email, None, None)?,
        };

        user.check_otp_rate_limit()?;

        let mut attempts = 0;
        loop {
            let otp = OtpCode::new();
            let expires_at = otp.expires_at();

            match user.add_otp_code(otp) {
                Ok(_) => {
                    self.user_repository.save(&user).await?;

                    return Ok(OtpData { expires_at });
                }
                Err(UserError::Otp(OtpError::DuplicateOtpCode)) => {
                    attempts += 1;
                    if attempts >= MAX_ATTEMPTS {
                        return Err(SendOtpError::User(UserError::Otp(
                            OtpError::DuplicateOtpCode,
                        )));
                    }

                    continue;
                }
                Err(e) => Err(e),
            }?;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OtpData {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendOtpError {
    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
