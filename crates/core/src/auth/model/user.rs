use std::{fmt, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use url::Url;
use uuid::Uuid;

use crate::{
    auth::{OtpCode, SocialAccount, UserError},
    common::UuidGenerator,
};

const OTP_RATE_LIMIT_COUNT: usize = 3;
const OTP_RATE_LIMIT_DURATION: u8 = 10;

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: EmailAddress,
    verified: bool,
    display_name: Option<String>,
    image_url: Option<Url>,
    social_accounts: Vec<SocialAccount>,
    otp_codes: Vec<OtpCode>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        display_name: Option<String>,
        image_url: Option<String>,
    ) -> Result<Self, UserError> {
        Self::new_with_generator(email, display_name, image_url, None)
    }

    pub fn new_with_generator(
        email: String,
        display_name: Option<String>,
        image_url: Option<String>,
        uuid_generator: Option<UuidGenerator>,
    ) -> Result<Self, UserError> {
        let email = email.parse()?;
        let image_url = image_url.as_deref().map(FromStr::from_str).transpose()?;

        let now = Utc::now();
        let uuid_generator =
            uuid_generator.unwrap_or_else(|| UuidGenerator::new().with_timestamp(now));

        Ok(Self {
            id: uuid_generator.generate().into(),
            email,
            verified: false,
            display_name,
            image_url,
            social_accounts: Vec::new(),
            otp_codes: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn add_social_account(&mut self, value: SocialAccount) -> Result<(), UserError> {
        if self
            .social_accounts
            .iter()
            .any(|e| e.provider() == value.provider() && e.sub() == value.sub())
        {
            return Err(UserError::DuplicateAccount(
                value.provider().to_owned(),
                value.sub().into(),
            ));
        }

        self.social_accounts.push(value);

        Ok(())
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
            .ok_or(UserError::InvalidOtpCode)?;

        opt_code.use_up()?;

        self.verified = true;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_values(
        id: UserId,
        email: EmailAddress,
        verified: bool,
        display_name: Option<String>,
        image_url: Option<Url>,
        social_accounts: Vec<SocialAccount>,
        otp_codes: Vec<OtpCode>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            verified,
            display_name,
            image_url,
            social_accounts,
            otp_codes,
            created_at,
            updated_at,
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

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn image_url(&self) -> Option<&Url> {
        self.image_url.as_ref()
    }

    pub fn social_accounts(&self) -> &[SocialAccount] {
        &self.social_accounts
    }

    pub fn otp_codes(&self) -> &[OtpCode] {
        &self.otp_codes
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
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
