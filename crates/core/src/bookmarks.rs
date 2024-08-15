use std::collections::HashMap;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    common::Paginated,
    scraper::{self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin},
    Tag,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub sort_index: u32,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Clone, Debug)]
pub struct BookmarkExtractorOptions<'a> {
    pub title_queries: Vec<ExtractorQuery<'a>>,
    pub published_queries: Vec<ExtractorQuery<'a>>,
    pub author_queries: Vec<ExtractorQuery<'a>>,
    pub thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExtractedBookmark {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub published: Option<String>,
    pub author: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProcessedBookmark {
    pub title: String,
    pub thumbnail: Option<Url>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

pub struct BookmarkPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<()>>,
    pub extractors:
        HashMap<&'static str, ExtractorPlugin<BookmarkExtractorOptions<'a>, ExtractedBookmark>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedBookmark, (), ProcessedBookmark>>,
}

#[async_trait::async_trait]
pub trait BookmarksRepository: Send + Sync {
    async fn find_many_bookmarks(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<BookmarksFindManyFilters>,
    ) -> Result<Paginated<Bookmark>, Error>;

    async fn find_one_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<Bookmark, Error>;

    async fn create_bookmark(&self, data: BookmarksCreateData) -> Result<Bookmark, Error>;

    async fn update_bookmark(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error>;

    async fn delete_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error>;
}

#[derive(Clone, Debug, Default)]
pub struct BookmarksFindManyFilters {
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct BookmarksCreateData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct BookmarksUpdateData {
    pub sort_index: Option<u32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
