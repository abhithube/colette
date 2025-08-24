use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use uuid::Uuid;

use crate::filter::{BooleanOp, DateOp, NumberOp, TextOp};

#[derive(Debug, Clone)]
pub struct Entry {
    id: EntryId,
    read_status: ReadStatus,
    user_id: UserId,
}

impl Entry {
    pub fn id(&self) -> EntryId {
        self.id
    }

    pub fn read_status(&self) -> &ReadStatus {
        &self.read_status
    }

    pub fn mark_as_read(&mut self) -> Result<(), EntryError> {
        match self.read_status {
            ReadStatus::Unread => {
                self.read_status = ReadStatus::Read(Utc::now());

                Ok(())
            }
            ReadStatus::Read(_) => Err(EntryError::AlreadyRead(self.id.0)),
        }
    }

    pub fn mark_as_unread(&mut self) -> Result<(), EntryError> {
        match self.read_status {
            ReadStatus::Unread => Err(EntryError::AlreadyUnread(self.id.0)),
            ReadStatus::Read(_) => {
                self.read_status = ReadStatus::Unread;

                Ok(())
            }
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn from_unchecked(id: Uuid, read_status: ReadStatus, user_id: Uuid) -> Self {
        Self {
            id: EntryId(id),
            read_status,
            user_id: user_id.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId(Uuid);

impl EntryId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for EntryId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub enum ReadStatus {
    Unread,
    Read(DateTime<Utc>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryFilter {
    Text {
        field: EntryTextField,
        op: TextOp,
    },
    Number {
        field: EntryNumberField,
        op: NumberOp,
    },
    Boolean {
        field: EntryBooleanField,
        op: BooleanOp,
    },
    Date {
        field: EntryDateField,
        op: DateOp,
    },

    And(Vec<EntryFilter>),
    Or(Vec<EntryFilter>),
    Not(Box<EntryFilter>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryNumberField {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryBooleanField {
    HasRead,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryDateField {
    PublishedAt,
}

#[derive(Debug, thiserror::Error)]
pub enum EntryError {
    #[error("entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("entry {0} already marked as read")]
    AlreadyRead(Uuid),

    #[error("entry {0} already marked as unread")]
    AlreadyUnread(Uuid),
}
