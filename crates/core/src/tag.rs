use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
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
        cursor: Option<String>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Paginated<Tag>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct TagFindManyFilters {
    pub tag_type: TagType,
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
