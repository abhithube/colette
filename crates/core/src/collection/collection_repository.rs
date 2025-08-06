use uuid::Uuid;

use super::{Collection, Error};
use crate::bookmark::BookmarkFilter;

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CollectionById>, Error>;

    async fn insert(&self, params: CollectionInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: CollectionUpdateParams) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CollectionById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct CollectionInsertParams {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct CollectionUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}
