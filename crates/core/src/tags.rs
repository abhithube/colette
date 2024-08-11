use uuid::Uuid;

use crate::common::Paginated;

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
pub trait TagsRepository: Send + Sync {
    async fn find_many_tags(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: TagsFindManyFilters,
    ) -> Result<Paginated<Tag>, Error>;

    async fn find_one_tag(&self, id: Uuid, profile_id: Uuid) -> Result<Tag, Error>;

    async fn create_tag(&self, data: TagsCreateData) -> Result<Tag, Error>;

    async fn update_tag(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: TagsUpdateData,
    ) -> Result<Tag, Error>;

    async fn delete_tag(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct TagsFindManyFilters {
    pub tag_type: TagType,
}

#[derive(Clone, Debug)]
pub struct TagsCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct TagsUpdateData {
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
