use chrono::{DateTime, Utc};
use colette_util::uuid_generate_ts;
use uuid::Uuid;

use crate::auth::UserId;

pub const PAT_VALUE_LENGTH: usize = 32;
pub const PAT_TITLE_MAX_LENGTH: usize = 50;

#[derive(Debug, Clone)]
pub struct PersonalAccessToken {
    id: PatId,
    lookup_hash: LookupHash,
    verification_hash: VerificationHash,
    title: PatTitle,
    preview: PatPreview,
    user_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PersonalAccessToken {
    pub fn new(
        lookup_hash: LookupHash,
        verification_hash: VerificationHash,
        title: PatTitle,
        preview: PatPreview,
        user_id: UserId,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ts(now).into(),
            lookup_hash,
            verification_hash,
            title,
            preview,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> PatId {
        self.id
    }

    pub fn lookup_hash(&self) -> &LookupHash {
        &self.lookup_hash
    }

    pub fn verification_hash(&self) -> &VerificationHash {
        &self.verification_hash
    }

    pub fn title(&self) -> &PatTitle {
        &self.title
    }

    pub fn set_title(&mut self, value: PatTitle) {
        if self.title != value {
            self.title = value;
            self.updated_at = Utc::now()
        }
    }

    pub fn preview(&self) -> &PatPreview {
        &self.preview
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
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
        lookup_hash: String,
        verification_hash: String,
        title: String,
        preview: String,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: PatId(id),
            lookup_hash: LookupHash(lookup_hash),
            verification_hash: VerificationHash(verification_hash),
            title: PatTitle(title),
            preview: PatPreview(preview),
            user_id: user_id.into(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatByLookupHash {
    id: PatId,
    verification_hash: VerificationHash,
    user_id: UserId,
}

impl PatByLookupHash {
    pub fn from_unchecked(id: Uuid, verification_hash: String, user_id: Uuid) -> Self {
        Self {
            id: PatId(id),
            verification_hash: VerificationHash(verification_hash),
            user_id: user_id.into(),
        }
    }

    pub fn id(&self) -> PatId {
        self.id
    }

    pub fn verification_hash(&self) -> &VerificationHash {
        &self.verification_hash
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

#[derive(Debug, Clone)]
pub struct PatValue(String);

impl PatValue {
    pub fn new(value: String) -> Result<Self, PatError> {
        if value.len() != PAT_VALUE_LENGTH {
            return Err(PatError::InvalidValueLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct LookupHash(String);

impl LookupHash {
    pub fn new(value: String) -> Result<Self, PatError> {
        if value.is_empty() {
            return Err(PatError::EmptyLookupHash);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct VerificationHash(String);

impl VerificationHash {
    pub fn new(value: String) -> Result<Self, PatError> {
        if value.is_empty() {
            return Err(PatError::EmptyVerificationHash);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatTitle(String);

impl PatTitle {
    pub fn new(value: String) -> Result<Self, PatError> {
        if value.is_empty() || value.len() > PAT_TITLE_MAX_LENGTH {
            return Err(PatError::InvalidTitleLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct PatPreview(String);

impl PatPreview {
    pub fn new(value: &PatValue) -> Self {
        let raw = value.as_inner();

        let formatted = format!("{}...{}", &raw[0..8], &raw[raw.len() - 4..raw.len()]);

        Self(formatted)
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatId(Uuid);

impl PatId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for PatId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PatError {
    #[error("PAT title must be between 1 and {PAT_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error("PAT value must be {PAT_VALUE_LENGTH} characters long")]
    InvalidValueLength,

    #[error("lookup hash cannot be empty")]
    EmptyLookupHash,

    #[error("verification hash cannot be empty")]
    EmptyVerificationHash,

    #[error("PAT not found with ID: {0}")]
    NotFound(Uuid),
}
