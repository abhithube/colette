use chrono::{DateTime, Duration, Utc};

pub const OTP_CODE_LEN: usize = 6;
pub const OTP_CODE_EXPIRATION_MIN: u8 = 10;

#[derive(Debug, Clone)]
pub struct OtpCode {
    code: CodeValue,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OtpCode {
    pub fn new(code: CodeValue) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(OTP_CODE_EXPIRATION_MIN as i64);

        Self {
            code,
            expires_at,
            used_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn code(&self) -> &CodeValue {
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
            return Err(OtpError::AlreadyUsedCode);
        }

        let now = Utc::now();
        if self.expires_at < now {
            return Err(OtpError::InvalidCode);
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
            code: CodeValue(code),
            expires_at,
            used_at,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeValue(String);

impl CodeValue {
    pub fn new(value: String) -> Result<Self, OtpError> {
        if value.len() != OTP_CODE_LEN {
            return Err(OtpError::InvalidLength);
        }

        Ok(CodeValue(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OtpError {
    #[error("OTP code must be exactly {OTP_CODE_LEN} characters long")]
    InvalidLength,

    #[error("invalid OTP code")]
    InvalidCode,

    #[error("already used OTP code")]
    AlreadyUsedCode,
}
