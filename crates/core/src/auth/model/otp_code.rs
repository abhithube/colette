use chrono::{DateTime, Duration, Utc};

use crate::common::NumericCodeGenerator;

pub const OTP_CODE_LEN: u8 = 6;
pub const OTP_CODE_EXPIRATION_MIN: u8 = 10;

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
        let code = NumericCodeGenerator::generate(OTP_CODE_LEN);

        let now = Utc::now();
        let expires_at = now + Duration::minutes(OTP_CODE_EXPIRATION_MIN as i64);

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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn used_at(&self) -> Option<DateTime<Utc>> {
        self.used_at
    }

    pub fn use_up(&mut self) -> Result<(), OtpError> {
        if self.used_at.is_some() {
            return Err(OtpError::AlreadyUsedOtpCode);
        }

        let now = Utc::now();
        if self.expires_at < now {
            return Err(OtpError::InvalidOtpCode);
        }

        self.used_at = Some(now);
        self.updated_at = now;

        Ok(())
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn from_unchecked(
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
}

#[derive(Debug, thiserror::Error)]
pub enum OtpError {
    #[error("invalid OTP code")]
    InvalidOtpCode,

    #[error("already used OTP code")]
    AlreadyUsedOtpCode,
}
