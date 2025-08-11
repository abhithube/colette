use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{pagination::Cursor, user::UserId};

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn authorize(&self, user_id: UserId) -> Result<(), ApiKeyError> {
        if self.user_id != user_id {
            return Err(ApiKeyError::Forbidden(user_id));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyId(Uuid);

impl ApiKeyId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for ApiKeyId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for ApiKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for ApiKey {
    type Data = ApiKeyCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(UserId),
}
