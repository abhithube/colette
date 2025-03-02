use uuid::Uuid;

use super::{BookmarkFilter, Collection, Cursor, Error};
use crate::{Bookmark, bookmark, common::IdParams};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find_collections(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<Collection>, Error>;

    async fn create_collection(&self, data: CollectionCreateData) -> Result<Uuid, Error>;

    async fn update_collection(
        &self,
        params: IdParams,
        data: CollectionUpdateData,
    ) -> Result<(), Error>;

    async fn delete_collection(&self, params: IdParams) -> Result<(), Error>;

    async fn find_bookmarks(
        &self,
        params: CollectionBookmarkFindParams,
    ) -> Result<Vec<Bookmark>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct CollectionCreateData {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}

#[derive(Debug, Clone)]
pub struct CollectionBookmarkFindParams {
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<bookmark::Cursor>,
}
