use chrono::{DateTime, Utc};
use url::Url;

use crate::{
    auth::UserId,
    bookmark::{Bookmark, BookmarkDto, BookmarkFilter, BookmarkId},
    common::RepositoryError,
    tag::TagId,
};

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<BookmarkDto>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: BookmarkId,
        user_id: UserId,
    ) -> Result<Option<Bookmark>, RepositoryError>;

    async fn save(&self, data: &Bookmark) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: BookmarkId, user_id: UserId) -> Result<(), RepositoryError>;

    async fn set_archived_path(
        &self,
        bookmark_id: BookmarkId,
        archived_path: Option<String>,
    ) -> Result<(), RepositoryError>;

    async fn import(&self, params: ImportBookmarksParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct BookmarkFindParams {
    pub user_id: UserId,
    pub id: Option<BookmarkId>,
    pub filter: Option<BookmarkFilter>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
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
