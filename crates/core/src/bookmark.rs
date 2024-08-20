use std::collections::HashMap;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable},
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
    pub collection_id: Option<Uuid>,
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
pub trait BookmarkRepository:
    Findable<Params = IdParams, Output = Result<Bookmark, Error>>
    + Creatable<Data = BookmarkCreateData, Output = Result<Bookmark, Error>>
    + Updatable<Params = IdParams, Data = BookmarkUpdateData, Output = Result<Bookmark, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Paginated<Bookmark>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkFindManyFilters {
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct BookmarkCreateData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub collection_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct BookmarkUpdateData {
    pub sort_index: Option<u32>,
    pub collection_id: Option<Option<Uuid>>,
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
