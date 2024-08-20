use uuid::Uuid;

use crate::common::{Creatable, IdParams, Paginated};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum TagType {
    All,
    Bookmarks,
    Feeds,
}

#[async_trait::async_trait]
pub trait TagRepository:
    Creatable<Data = TagCreateData, Output = Result<Tag, Error>> + Send + Sync
{
    async fn find_many(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Paginated<Tag>, Error>;

    async fn find_one(&self, params: IdParams) -> Result<Tag, Error>;

    async fn update(&self, params: IdParams, data: TagUpdateData) -> Result<Tag, Error>;

    async fn delete(&self, params: IdParams) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct TagFindManyFilters {
    pub tag_type: TagType,
}

#[derive(Clone, Debug)]
pub struct TagCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
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
