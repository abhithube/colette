use chrono::{DateTime, Utc};
use dyn_clone::DynClone;
use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, NonEmptyVec, Paginated, Updatable,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartFeed {
    pub id: Uuid,
    pub title: String,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmartFeedCreate {
    pub title: NonEmptyString,
    pub filters: Option<NonEmptyVec<SmartFeedFilter>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartFeedUpdate {
    pub title: Option<NonEmptyString>,
    pub filters: Option<NonEmptyVec<SmartFeedFilter>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SmartFeedFilter {
    Link(TextOperation),
    Title(TextOperation),
    PublishedAt(DateOperation),
    Description(TextOperation),
    Author(TextOperation),
    HasRead(BooleanOperation),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TextOperation {
    Equals(String),
    DoesNotEqual(String),
    Contains(String),
    DoesNotContain(String),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BooleanOperation {
    pub value: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DateOperation {
    Equals(DateTime<Utc>),
    GreaterThan(DateTime<Utc>),
    LessThan(DateTime<Utc>),
    InLast(i64),
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Clone)]
pub struct SmartFeedService {
    repository: Box<dyn SmartFeedRepository>,
}

impl SmartFeedService {
    pub fn new(repository: Box<dyn SmartFeedRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_smart_feeds(&self, user_id: Uuid) -> Result<Paginated<SmartFeed>, Error> {
        let feeds = self
            .repository
            .find(SmartFeedFindParams {
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: feeds,
            ..Default::default()
        })
    }

    pub async fn get_smart_feed(&self, id: Uuid, user_id: Uuid) -> Result<SmartFeed, Error> {
        let mut smart_feeds = self
            .repository
            .find(SmartFeedFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if smart_feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(smart_feeds.swap_remove(0))
    }

    pub async fn create_smart_feed(
        &self,
        data: SmartFeedCreate,
        user_id: Uuid,
    ) -> Result<SmartFeed, Error> {
        let id = self
            .repository
            .create(SmartFeedCreateData {
                title: data.title.into(),
                filters: data.filters.map(Vec::from),
                user_id,
            })
            .await?;

        self.get_smart_feed(id, user_id).await
    }

    pub async fn update_smart_feed(
        &self,
        id: Uuid,
        data: SmartFeedUpdate,
        user_id: Uuid,
    ) -> Result<SmartFeed, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_smart_feed(id, user_id).await
    }

    pub async fn delete_smart_feed(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

pub trait SmartFeedRepository:
    Findable<Params = SmartFeedFindParams, Output = Result<Vec<SmartFeed>, Error>>
    + Creatable<Data = SmartFeedCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = SmartFeedUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + DynClone
{
}

dyn_clone::clone_trait_object!(SmartFeedRepository);

#[derive(Clone, Debug, Default)]
pub struct SmartFeedFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct SmartFeedCreateData {
    pub title: String,
    pub filters: Option<Vec<SmartFeedFilter>>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct SmartFeedUpdateData {
    pub title: Option<String>,
    pub filters: Option<Vec<SmartFeedFilter>>,
}

impl From<SmartFeedUpdate> for SmartFeedUpdateData {
    fn from(value: SmartFeedUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            filters: value.filters.map(Vec::from),
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
