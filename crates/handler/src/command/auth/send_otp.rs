use chrono::{DateTime, Utc};
use colette_authentication::{
    CodeValue, OTP_CODE_EXPIRATION_MIN, OTP_CODE_LEN, OTP_MAX_ATTEMPTS, OtpCode, User, UserError,
    UserRepository,
};
use colette_common::RepositoryError;
use colette_crypto::CodeGenerator;
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

pub struct SendOtpHandler<UR: UserRepository, SC: SmtpClient, CG: CodeGenerator> {
    user_repository: UR,
    smtp_client: SC,
    otp_code_generator: CG,
}

impl<UR: UserRepository, SC: SmtpClient, CG: CodeGenerator> SendOtpHandler<UR, SC, CG> {
    pub fn new(user_repository: UR, smtp_client: SC, otp_code_generator: CG) -> Self {
        Self {
            user_repository,
            smtp_client,
            otp_code_generator,
        }
    }
}

impl<UR: UserRepository, SC: SmtpClient, CG: CodeGenerator> Handler<SendOtpCommand>
    for SendOtpHandler<UR, SC, CG>
{
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

        let mut attempts = 0;
        let otp_code = loop {
            let code = self.otp_code_generator.generate(OTP_CODE_LEN);
            let code = String::from_utf8_lossy(&code).into_owned();
            let code = match CodeValue::new(code) {
                Ok(code) => code,
                Err(e) => return Err(SendOtpError::User(UserError::Otp(e))),
            };

            let otp_code = OtpCode::new(code);

            if user.otp_codes().iter().any(|e| e.code() == otp_code.code()) {
                attempts += 1;
                if attempts >= OTP_MAX_ATTEMPTS {
                    return Err(SendOtpError::User(UserError::TooManyOtpCodes));
                }

                continue;
            }

            user.add_otp_code(otp_code.clone())?;

            break otp_code;
        };

        let to_address = if let Some(display_name) = user.display_name() {
            user.email().to_display(display_name.as_inner())
        } else {
            user.email().email()
        };

        let body = OTP_EMAIL_BODY
            .replace(OTP_CODE_PLACEHOLDER, otp_code.code().as_inner())
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
