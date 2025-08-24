use chrono::{DateTime, Utc};
use colette_core::{
    User,
    auth::{OTP_CODE_EXPIRATION_MIN, UserError, UserRepository},
    common::RepositoryError,
};
use colette_smtp::{SmtpClient, SmtpEmail};
use email_address::EmailAddress;

use crate::Handler;

const OTP_EMAIL_SUBJECT: &str = "Verify your identity";
const OTP_EMAIL_BODY: &str = include_str!("./template/otp.txt");

const OTP_CODE_PLACEHOLDER: &str = "{{OTP_CODE}}";
const OTP_EXPIRATION_MIN_PLACEHOLDER: &str = "{{OTP_EXPIRATION_MIN}}";
const USER_EMAIL_PLACEHOLDER: &str = "{{USER_EMAIL}}";

#[derive(Debug, Clone)]
pub struct SendOtpCommand {
    pub email: String,
}

pub struct SendOtpHandler<UR: UserRepository, SC: SmtpClient> {
    user_repository: UR,
    smtp_client: SC,
}

impl<UR: UserRepository, SC: SmtpClient> SendOtpHandler<UR, SC> {
    pub fn new(user_repository: UR, smtp_client: SC) -> Self {
        Self {
            user_repository,
            smtp_client,
        }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository, SC: SmtpClient> Handler<SendOtpCommand> for SendOtpHandler<UR, SC> {
    type Response = OtpData;
    type Error = SendOtpError;

    async fn handle(&self, cmd: SendOtpCommand) -> Result<Self::Response, Self::Error> {
        let email = cmd
            .email
            .parse::<EmailAddress>()
            .map_err(UserError::InvalidEmail)?;

        let mut user = match self.user_repository.find_by_email(email.clone()).await? {
            Some(user) => user,
            None => User::new(email, None, None),
        };

        user.check_otp_rate_limit()?;

        let otp_code = user.generate_otp_code()?;

        let to_address = if let Some(display_name) = user.display_name() {
            user.email().to_display(display_name.as_inner())
        } else {
            user.email().email()
        };

        let body = OTP_EMAIL_BODY
            .replace(OTP_CODE_PLACEHOLDER, otp_code.code())
            .replace(
                OTP_EXPIRATION_MIN_PLACEHOLDER,
                &OTP_CODE_EXPIRATION_MIN.to_string(),
            )
            .replace(USER_EMAIL_PLACEHOLDER, user.email().as_str());

        self.smtp_client
            .send(SmtpEmail {
                to_address,
                subject: OTP_EMAIL_SUBJECT.into(),
                body,
            })
            .await?;

        self.user_repository.save(&user).await?;

        Ok(OtpData {
            expires_at: otp_code.expires_at(),
        })
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
    Smtp(#[from] colette_smtp::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
