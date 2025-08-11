use std::fmt;

use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: EmailAddress,
    pub display_name: Option<String>,
    pub image_url: Option<Url>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

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
