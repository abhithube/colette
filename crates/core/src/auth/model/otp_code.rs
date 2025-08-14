use chrono::{DateTime, Duration, Utc};

use crate::{auth::UserError, common::NumericCodeGenerator};

const OTP_LEN: u8 = 6;
const OTP_EXPIRATION_MIN: u8 = 10;

#[derive(Debug, Clone)]
pub struct OtpCode {
    code: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Default for OtpCode {
    fn default() -> Self {
        let code = NumericCodeGenerator::generate(OTP_LEN);

        let now = Utc::now();
        let expires_at = now + Duration::minutes(OTP_EXPIRATION_MIN as i64);

        Self {
            code: String::from_utf8_lossy(&code).into_owned(),
            expires_at,
            used_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl OtpCode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_values(
        code: String,
        expires_at: DateTime<Utc>,
        used_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            code,
            expires_at,
            used_at,
            created_at,
            updated_at,
        }
    }

    pub fn use_up(&mut self) -> Result<(), UserError> {
        if self.used_at.is_some() {
            return Err(UserError::AlreadyUsedOtpCode);
        }

        let now = Utc::now();
        if self.expires_at < now {
            return Err(UserError::InvalidOtpCode);
        }

        self.used_at = Some(now);

        Ok(())
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn used_at(&self) -> Option<DateTime<Utc>> {
        self.used_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
