use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};

use crate::auth::UserError;

#[derive(Debug, Clone)]
pub struct SocialAccount {
    provider: Provider,
    sub: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SocialAccount {
    pub fn new(provider: Provider, sub: String) -> Self {
        let now = Utc::now();

        Self {
            provider,
            sub,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_values(
        provider: Provider,
        sub: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            provider,
            sub,
            created_at,
            updated_at,
        }
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Google,
    Apple,
    Other(String),
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Google => "google",
            Self::Apple => "apple",
            Self::Other(provider) => provider,
        };

        write!(f, "{raw}")
    }
}

impl FromStr for Provider {
    type Err = UserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider = match s {
            "google" => Self::Google,
            "apple" => Self::Apple,
            _ => Self::Other(s.into()),
        };

        Ok(provider)
    }
}
