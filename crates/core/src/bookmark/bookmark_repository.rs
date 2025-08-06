use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use super::{Bookmark, Error};
use crate::bookmark::BookmarkFilter;

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BookmarkById>, Error>;

    async fn insert(&self, params: BookmarkInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: BookmarkUpdateParams) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error>;

    async fn link_tags(&self, params: BookmarkLinkTagParams) -> Result<(), Error>;

    async fn import(&self, params: ImportBookmarksParams) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindParams {
    pub id: Option<Uuid>,
    pub filter: Option<BookmarkFilter>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkById {
    pub id: Uuid,
    pub thumbnail_url: Option<Url>,
    pub archived_path: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct BookmarkInsertParams {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub user_id: Uuid,
    pub upsert: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct BookmarkLinkTagParams {
    pub bookmark_id: Uuid,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ImportBookmarksParams {
    pub bookmark_items: Vec<BookmarkBatchItem>,
    pub tag_titles: Vec<String>,
    pub user_id: Uuid,
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
