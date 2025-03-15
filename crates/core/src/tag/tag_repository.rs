use uuid::Uuid;

use super::{Error, Tag, TagType};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, Error>;

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Tag>, Error>;

    async fn save(&self, data: &Tag, upsert: Option<TagUpsertType>) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct TagFindParams {
    pub ids: Option<Vec<Uuid>>,
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}

pub enum TagUpsertType {
    Id,
    Title,
}
