use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::RepositoryError;
use url::Url;

use crate::{Bookmark, BookmarkId};

pub trait BookmarkRepository: Sync {
    fn find_by_id(
        &self,
        id: BookmarkId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Bookmark>, RepositoryError>> + Send;

    fn save(&self, data: &Bookmark) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn delete_by_id(
        &self,
        id: BookmarkId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn set_archived_path(
        &self,
        bookmark_id: BookmarkId,
        archived_path: Option<String>,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn import(
        &self,
        params: ImportBookmarksParams,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

#[derive(Debug, Clone)]
pub struct ImportBookmarksParams {
    pub bookmark_items: Vec<BookmarkBatchItem>,
    pub tag_titles: Vec<String>,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct BookmarkBatchItem {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tag_titles: Vec<String>,
}
