use uuid::Uuid;

use super::{Cursor, Error, Tag, TagType};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find_tags(&self, params: TagFindParams) -> Result<Vec<Tag>, Error>;

    async fn find_tag_by_id(
        &self,
        tx: &dyn Transaction,
        params: TagFindByIdParams,
    ) -> Result<TagById, Error>;

    async fn create_tag(&self, params: TagCreateParams) -> Result<(), Error>;

    async fn update_tag(&self, tx: &dyn Transaction, params: TagUpdateParams) -> Result<(), Error>;

    async fn delete_tag(&self, tx: &dyn Transaction, params: TagDeleteParams) -> Result<(), Error>;
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
pub struct TagFindByIdParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagCreateParams {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TagDeleteParams {
    pub id: Uuid,
}
