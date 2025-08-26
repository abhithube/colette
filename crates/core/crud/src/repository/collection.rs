use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::{BookmarkFilter, Collection, CollectionId};

pub trait CollectionRepository: Sync {
    fn find_by_id(
        &self,
        id: CollectionId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Collection>, RepositoryError>> + Send;

    fn save(&self, data: &Collection) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn delete_by_id(
        &self,
        id: CollectionId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

#[derive(Debug, Clone)]
pub struct CollectionInsertParams {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct CollectionUpdateParams {
    pub id: CollectionId,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}
