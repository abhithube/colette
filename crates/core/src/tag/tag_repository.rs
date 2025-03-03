use uuid::Uuid;

use super::{Cursor, Error, Tag, TagType};
use crate::common::IdParams;

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find_tags(&self, params: TagFindParams) -> Result<Vec<Tag>, Error>;

    async fn find_tag_by_id(&self, id: Uuid) -> Result<TagById, Error>;

    async fn create_tag(&self, data: TagCreateData) -> Result<Uuid, Error>;

    async fn update_tag(&self, params: IdParams, data: TagUpdateData) -> Result<(), Error>;

    async fn delete_tag(&self, params: IdParams) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct TagById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagFindParams {
    pub ids: Option<Vec<Uuid>>,
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct TagCreateData {
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdateData {
    pub title: Option<String>,
}
