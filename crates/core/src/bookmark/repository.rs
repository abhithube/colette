use chrono::{DateTime, Utc};
use url::Url;

use crate::{
    bookmark::{Bookmark, BookmarkFilter, BookmarkId},
    common::RepositoryError,
    tag::TagId,
    user::UserId,
};

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, RepositoryError>;

    async fn find_by_id(&self, id: BookmarkId) -> Result<Option<Bookmark>, RepositoryError> {
        let mut bookmarks = self
            .find(BookmarkFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Ok(None);
        }

        Ok(Some(bookmarks.swap_remove(0)))
    }

    async fn insert(&self, params: BookmarkInsertParams) -> Result<BookmarkId, RepositoryError>;

    async fn update(&self, params: BookmarkUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: BookmarkId) -> Result<(), RepositoryError>;

    async fn set_archived_path(
        &self,
        bookmark_id: BookmarkId,
        archived_path: Option<String>,
    ) -> Result<(), RepositoryError>;

    async fn link_tags(&self, params: BookmarkLinkTagParams) -> Result<(), RepositoryError>;

    async fn import(&self, params: ImportBookmarksParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindParams {
    pub id: Option<BookmarkId>,
    pub filter: Option<BookmarkFilter>,
    pub tags: Option<Vec<TagId>>,
    pub user_id: Option<UserId>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkInsertParams {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub user_id: UserId,
    pub upsert: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkUpdateParams {
    pub id: BookmarkId,
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct BookmarkLinkTagParams {
    pub bookmark_id: BookmarkId,
    pub tag_ids: Vec<TagId>,
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
