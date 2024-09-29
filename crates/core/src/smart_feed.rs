use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartFeed {
    pub id: Uuid,
    pub title: String,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmartFeedCreate {
    pub title: NonEmptyString,
    pub filters: Option<Vec<SmartFeedFilter>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartFeedUpdate {
    pub title: Option<NonEmptyString>,
    pub filters: Option<Vec<SmartFeedFilter>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SmartFeedFilter {
    Link {
        operation: TextOperation,
        negated: bool,
    },
    Title {
        operation: TextOperation,
        negated: bool,
    },
    PublishedAt {
        operation: DateOperation,
    },
    Description {
        operation: TextOperation,
        negated: bool,
    },
    Author {
        operation: TextOperation,
        negated: bool,
    },
    HasRead(bool),
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum TextOperation {
    #[default]
    Equals,
    Contains,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum DateOperation {
    #[default]
    Equals,
    GreaterThan,
    LessThan,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

#[async_trait::async_trait]
pub trait SmartFeedRepository:
    Findable<Params = IdParams, Output = Result<SmartFeed, Error>>
    + Creatable<Data = SmartFeedCreateData, Output = Result<SmartFeed, Error>>
    + Updatable<Params = IdParams, Data = SmartFeedUpdateData, Output = Result<SmartFeed, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<SmartFeed>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct SmartFeedCreateData {
    pub title: String,
    pub filters: Option<Vec<SmartFeedFilter>>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct SmartFeedUpdateData {
    pub title: Option<String>,
    pub filters: Option<Vec<SmartFeedFilter>>,
}

impl From<SmartFeedUpdate> for SmartFeedUpdateData {
    fn from(value: SmartFeedUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.into()),
            filters: value.filters,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("smart feed not found with id: {0}")]
    NotFound(Uuid),

    #[error("smart feed already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
