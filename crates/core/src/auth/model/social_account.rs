use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SocialAccount {
    provider: Provider,
    sub: Sub,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SocialAccount {
    pub fn new(provider: Provider, sub: Sub) -> Self {
        let now = Utc::now();

        Self {
            provider,
            sub,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn sub(&self) -> &Sub {
        &self.sub
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn from_unchecked(
        provider: String,
        sub: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            provider: provider.parse().unwrap(),
            sub: Sub(sub),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Google,
    Apple,
    Custom(CustomProvider),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomProvider(String);

impl CustomProvider {
    pub fn new(value: String) -> Result<Self, SocialAccountError> {
        if value.is_empty() {
            return Err(SocialAccountError::EmptyProvider);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Google => "google",
            Self::Apple => "apple",
            Self::Custom(provider) => provider.as_inner(),
        };

        write!(f, "{raw}")
    }
}

impl FromStr for Provider {
    type Err = SocialAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider = match s {
            "google" => Self::Google,
            "apple" => Self::Apple,
            _ => Self::Custom(CustomProvider::new(s.to_owned())?),
        };

        Ok(provider)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sub(String);

impl Sub {
    pub fn new(value: String) -> Result<Self, SocialAccountError> {
        if value.is_empty() {
            return Err(SocialAccountError::EmptySub);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_inner())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SocialAccountError {
    #[error("provider cannot be empty")]
    EmptyProvider,

    #[error("sub cannot be empty")]
    EmptySub,
}
