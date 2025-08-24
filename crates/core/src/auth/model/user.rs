use chrono::{DateTime, Duration, Utc};
use colette_util::uuid_generate_ts;
use email_address::EmailAddress;
use url::Url;
use uuid::Uuid;

use crate::auth::{OtpCode, OtpError, Provider, SocialAccount, SocialAccountError};

pub const USER_DISPLAY_NAME_MAX_LENGTH: usize = 50;
pub const OTP_CODE_MAX_ATTEMPTS: i8 = 5;
pub const OTP_RATE_LIMIT_COUNT: usize = 3;
pub const OTP_RATE_LIMIT_DURATION: u8 = 10;
pub const PAT_MAX_COUNT: usize = 10;

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: EmailAddress,
    verified: bool,
    display_name: Option<DisplayName>,
    image_url: Option<Url>,
    social_accounts: Vec<SocialAccount>,
    otp_codes: Vec<OtpCode>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: EmailAddress,
        display_name: Option<DisplayName>,
        image_url: Option<Url>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ts(now).into(),
            email,
            verified: false,
            display_name,
            image_url,
            social_accounts: Vec::new(),
            otp_codes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    pub fn verified(&self) -> bool {
        self.verified
    }

    pub fn display_name(&self) -> Option<&DisplayName> {
        self.display_name.as_ref()
    }

    pub fn image_url(&self) -> Option<&Url> {
        self.image_url.as_ref()
    }

    pub fn otp_codes(&self) -> &[OtpCode] {
        &self.otp_codes
    }

    pub fn check_otp_rate_limit(&self) -> Result<(), UserError> {
        if self.otp_codes.is_empty() {
            return Ok(());
        }

        let time = Utc::now() - Duration::minutes(OTP_RATE_LIMIT_DURATION as i64);

        if self
            .otp_codes
            .iter()
            .rev()
            .take(OTP_RATE_LIMIT_COUNT)
            .all(|e| e.created_at() >= time)
        {
            return Err(UserError::TooManyOtpCodes);
        }

        Ok(())
    }

    pub fn generate_otp_code(&mut self) -> Result<OtpCode, UserError> {
        let mut attempts = 0;
        loop {
            let otp_code: OtpCode = OtpCode::new();

            if self.otp_codes.iter().any(|e| e.code() == otp_code.code()) {
                attempts += 1;
                if attempts >= OTP_CODE_MAX_ATTEMPTS {
                    return Err(UserError::DuplicateOtpCode);
                }

                continue;
            }

            self.otp_codes.push(otp_code.clone());

            return Ok(otp_code);
        }
    }

    pub fn add_otp_code(&mut self, value: OtpCode) -> Result<(), UserError> {
        if self.otp_codes.iter().any(|e| e.code() == value.code()) {
            return Err(UserError::DuplicateOtpCode);
        }

        self.otp_codes.push(value);

        Ok(())
    }

    pub fn use_otp_code(&mut self, code: String) -> Result<(), UserError> {
        let opt_code = self
            .otp_codes
            .iter_mut()
            .find(|e| e.code() == code)
            .ok_or(OtpError::InvalidOtpCode)?;

        opt_code.use_up()?;

        self.verified = true;

        Ok(())
    }

    pub fn social_accounts(&self) -> &[SocialAccount] {
        &self.social_accounts
    }

    pub fn add_social_account(&mut self, value: SocialAccount) -> Result<(), UserError> {
        if self
            .social_accounts
            .iter()
            .any(|e| e.provider() == value.provider() && e.sub() == value.sub())
        {
            return Err(UserError::DuplicateAccount(
                value.provider().to_owned(),
                value.sub().as_inner().to_owned(),
            ));
        }

        self.social_accounts.push(value);

        Ok(())
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_unchecked(
        id: Uuid,
        email: String,
        verified: bool,
        display_name: Option<String>,
        image_url: Option<Url>,
        otp_codes: Vec<OtpCode>,
        social_accounts: Vec<SocialAccount>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: UserId(id),
            email: email.parse().unwrap(),
            verified,
            display_name: display_name.map(DisplayName),
            image_url,
            otp_codes,
            social_accounts,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: String) -> Result<Self, UserError> {
        if value.is_empty() || value.len() > USER_DISPLAY_NAME_MAX_LENGTH {
            return Err(UserError::InvalidDisplayNameLength);
        }

        Ok(Self(value))
    }

    pub fn new_truncating(value: String) -> Result<Self, UserError> {
        if value.is_empty() {
            return Err(UserError::InvalidDisplayNameLength);
        }

        let truncated = if value.len() > USER_DISPLAY_NAME_MAX_LENGTH {
            value[0..USER_DISPLAY_NAME_MAX_LENGTH].to_owned()
        } else {
            value
        };

        Ok(Self(truncated))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error(transparent)]
    InvalidEmail(#[from] email_address::Error),

    #[error(transparent)]
    InvalidImageUrl(#[from] url::ParseError),

    #[error("display name must be between 1 and {USER_DISPLAY_NAME_MAX_LENGTH} characters long")]
    InvalidDisplayNameLength,

    #[error("already connected to provider {0} with sub {1}")]
    DuplicateAccount(Provider, String),

    #[error("created too many OTP codes")]
    TooManyOtpCodes,

    #[error("duplicate OTP code")]
    DuplicateOtpCode,

    #[error("created too many PATs")]
    TooManyPats,

    #[error(transparent)]
    Otp(#[from] OtpError),

    #[error(transparent)]
    SocialAccount(#[from] SocialAccountError),
}
