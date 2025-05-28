use uuid::Uuid;

use super::{Error, Tag, TagType};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error>;

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Tag>, Error> {
        self.query(TagParams {
            ids: Some(ids),
            ..Default::default()
        })
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Tag>, Error> {
        Ok(self.find_by_ids(vec![id]).await?.into_iter().next())
    }

    async fn save(&self, data: &Tag) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct TagParams {
    pub ids: Option<Vec<Uuid>>,
    pub title: Option<String>,
    pub tag_type: Option<TagType>,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
    pub with_subscription_count: bool,
    pub with_bookmark_count: bool,
}
