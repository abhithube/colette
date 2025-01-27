use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct TagCreate {
    pub title: NonEmptyString,
}

#[derive(Clone, Debug, Default)]
pub struct TagUpdate {
    pub title: Option<NonEmptyString>,
}

#[derive(Clone, Debug, Default)]
pub struct TagListQuery {
    pub tag_type: TagType,
}

#[derive(Clone, Debug, Default)]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

pub struct TagService {
    repository: Box<dyn TagRepository>,
}

impl TagService {
    pub fn new(repository: impl TagRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_tags(
        &self,
        query: TagListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repository
            .find(TagFindParams {
                tag_type: query.tag_type,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: tags,
            ..Default::default()
        })
    }

    pub async fn get_tag(&self, id: Uuid, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .find(TagFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(tags.swap_remove(0))
    }

    pub async fn create_tag(&self, data: TagCreate, user_id: Uuid) -> Result<Tag, Error> {
        let id = self
            .repository
            .create(TagCreateData {
                title: data.title.into(),
                user_id,
            })
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

pub trait TagRepository:
    Findable<Params = TagFindParams, Output = Result<Vec<Tag>, Error>>
    + Creatable<Data = TagCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = TagUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Clone, Debug, Default)]
pub struct TagFindParams {
    pub id: Option<Uuid>,
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct TagCreateData {
    pub title: String,
    pub user_id: Uuid,
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
