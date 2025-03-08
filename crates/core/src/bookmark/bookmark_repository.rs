use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use super::{Bookmark, Cursor, Error, ProcessedBookmark};
use crate::{bookmark::BookmarkFilter, common::Transaction, tag::TagById};

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn find_bookmarks(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error>;

    async fn find_bookmark_by_id(
        &self,
        tx: &dyn Transaction,
        params: BookmarkFindByIdParams,
    ) -> Result<BookmarkById, Error>;

    async fn create_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkCreateParams,
    ) -> Result<(), Error>;

    async fn update_bookmark(
        &self,
        tx: Option<&dyn Transaction>,
        params: BookmarkUpdateParams,
    ) -> Result<(), Error>;

    async fn delete_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkDeleteParams,
    ) -> Result<(), Error>;

    async fn save_scraped(&self, params: BookmarkScrapedParams) -> Result<(), Error>;

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: BookmarkTagsLinkParams,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindParams {
    pub filter: Option<BookmarkFilter>,
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindByIdParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct BookmarkCreateParams {
    pub id: Uuid,
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub archived_path: Option<Option<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkDeleteParams {
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct BookmarkScrapedParams {
    pub url: Url,
    pub bookmark: ProcessedBookmark,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkTagsLinkParams {
    pub bookmark_id: Uuid,
    pub tags: Vec<TagById>,
}
