use uuid::Uuid;

use super::{BookmarkFilter, Collection, Cursor, Error};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find_collections(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<Collection>, Error>;

    async fn find_collection_by_id(
        &self,
        tx: &dyn Transaction,
        params: CollectionFindByIdParams,
    ) -> Result<CollectionById, Error>;

    async fn create_collection(&self, params: CollectionCreateParams) -> Result<(), Error>;

    async fn update_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionUpdateParams,
    ) -> Result<(), Error>;

    async fn delete_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionDeleteParams,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindByIdParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct CollectionCreateParams {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionDeleteParams {
    pub id: Uuid,
}
