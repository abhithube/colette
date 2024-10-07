use std::sync::Arc;

use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TagCreate {
    pub title: NonEmptyString,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TagUpdate {
    pub title: Option<NonEmptyString>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TagListQuery {
    pub tag_type: TagType,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

pub struct TagService {
    repository: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(repository: Arc<dyn TagRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        query: TagListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repository
            .list(profile_id, None, None, Some(query.into()))
            .await?;

        Ok(Paginated {
            data: tags,
            ..Default::default()
        })
    }

    pub async fn get(&self, id: Uuid, profile_id: Uuid) -> Result<Tag, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create(&self, data: TagCreate, profile_id: Uuid) -> Result<Tag, Error> {
        self.repository
            .create(TagCreateData {
                title: data.title.into(),
                profile_id,
            })
            .await
    }

    pub async fn update(&self, id: Uuid, data: TagUpdate, profile_id: Uuid) -> Result<Tag, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }
}

#[async_trait::async_trait]
pub trait TagRepository:
    Findable<Params = IdParams, Output = Result<Tag, Error>>
    + Creatable<Data = TagCreateData, Output = Result<Tag, Error>>
    + Updatable<Params = IdParams, Data = TagUpdateData, Output = Result<Tag, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Vec<Tag>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct TagFindManyFilters {
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
}

impl From<TagListQuery> for TagFindManyFilters {
    fn from(value: TagListQuery) -> Self {
        Self {
            tag_type: value.tag_type,
            feed_id: None,
            bookmark_id: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TagCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct TagUpdateData {
    pub title: Option<String>,
}

impl From<TagUpdate> for TagUpdateData {
    fn from(value: TagUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
