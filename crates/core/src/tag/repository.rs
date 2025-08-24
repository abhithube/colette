use crate::{
    auth::UserId,
    common::RepositoryError,
    tag::{Tag, TagId},
};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: TagId, user_id: UserId) -> Result<Option<Tag>, RepositoryError>;

    async fn save(&self, data: &Tag) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: TagId, user_id: UserId) -> Result<(), RepositoryError>;
}
