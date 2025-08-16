use std::{fmt, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use url::Url;
use uuid::Uuid;

use crate::{
    auth::{
        OtpCode, OtpError, PatError, PatId, PersonalAccessToken, Provider, SocialAccount,
        SocialAccountError, Sub,
    },
    common::UuidGenerator,
};

const DISPLAY_NAME_MAX_LENGTH: usize = 50;
const OTP_RATE_LIMIT_COUNT: usize = 3;
const OTP_RATE_LIMIT_DURATION: u8 = 10;
const MAX_PAT_COUNT: usize = 10;

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: EmailAddress,
    verified: bool,
    display_name: Option<DisplayName>,
    image_url: Option<Url>,
    social_accounts: Vec<SocialAccount>,
    otp_codes: Vec<OtpCode>,
    personal_access_tokens: Vec<PersonalAccessToken>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        display_name: Option<DisplayName>,
        image_url: Option<String>,
    ) -> Result<Self, UserError> {
        let email = email.parse()?;
        let image_url = image_url.as_deref().map(FromStr::from_str).transpose()?;

        let now = Utc::now();

        Ok(Self {
            id: UuidGenerator::new().with_timestamp(now).generate().into(),
            email,
            verified: false,
            display_name,
            image_url,
            social_accounts: Vec::new(),
            otp_codes: Vec::new(),
            personal_access_tokens: Vec::new(),
            created_at: now,
            updated_at: now,
        })
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
            .ok_or(UserError::Otp(OtpError::InvalidOtpCode))?;

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
                value.sub().to_owned(),
            ));
        }

        self.social_accounts.push(value);

        Ok(())
    }

    pub fn personal_access_tokens(&self) -> &[PersonalAccessToken] {
        &self.personal_access_tokens
    }

    pub fn get_personal_access_token(&mut self, id: PatId) -> Option<&mut PersonalAccessToken> {
        self.personal_access_tokens
            .iter_mut()
            .find(|e| e.id() == id)
    }

    pub fn add_personal_access_token(
        &mut self,
        value: PersonalAccessToken,
    ) -> Result<(), UserError> {
        if self.personal_access_tokens.len() == MAX_PAT_COUNT {
            return Err(UserError::TooManyPats);
        }

        self.personal_access_tokens.push(value);

        Ok(())
    }

    pub fn remove_personal_access_token(&mut self, id: PatId) -> Result<(), UserError> {
        let index = self
            .personal_access_tokens
            .iter()
            .position(|e| e.id() == id)
            .ok_or(UserError::Pat(PatError::NotFound(id)))?;

        self.personal_access_tokens.remove(index);

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
        personal_access_tokens: Vec<PersonalAccessToken>,
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
            personal_access_tokens,
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

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: String) -> Result<Self, UserError> {
        if value.is_empty() || value.len() > DISPLAY_NAME_MAX_LENGTH {
            return Err(UserError::InvalidDisplayNameLength);
        }

        Ok(Self(value))
    }

    pub fn new_truncating(value: String) -> Result<Self, UserError> {
        if value.is_empty() {
            return Err(UserError::InvalidDisplayNameLength);
        }

        let truncated = if value.len() > DISPLAY_NAME_MAX_LENGTH {
            value[0..DISPLAY_NAME_MAX_LENGTH].to_owned()
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

    #[error("display name must be between 1 and {DISPLAY_NAME_MAX_LENGTH} characters long")]
    InvalidDisplayNameLength,

    #[error("already connected to provider {0} with sub {1}")]
    DuplicateAccount(Provider, Sub),

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

    #[error(transparent)]
    Pat(#[from] PatError),
}
