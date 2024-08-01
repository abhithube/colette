use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{FindOneParams, Paginated, Session, PAGINATION_LIMIT},
    utils::scraper::{
        self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin, Scraper,
    },
    Tag,
};

#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug)]
pub struct CreateBookmark {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct UpdateBookmark {
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug)]
pub struct ListBookmarksParams {
    pub published_at: Option<DateTime<Utc>>,
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
        params: BookmarksFindManyParams,
    ) -> Result<Vec<Bookmark>, Error>;

    async fn find_one_bookmark(&self, params: FindOneParams) -> Result<Bookmark, Error>;

    async fn create_bookmark(&self, data: BookmarksCreateData) -> Result<Bookmark, Error>;

    async fn update_bookmark(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error>;

    async fn delete_bookmark(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct BookmarksService {
    repo: Arc<dyn BookmarksRepository>,
    scraper: Arc<dyn Scraper<ProcessedBookmark>>,
}

impl BookmarksService {
    pub fn new(
        repo: Arc<dyn BookmarksRepository>,
        scraper: Arc<dyn Scraper<ProcessedBookmark>>,
    ) -> Self {
        Self { repo, scraper }
    }

    pub async fn list(
        &self,
        params: ListBookmarksParams,
        session: Session,
    ) -> Result<Paginated<Bookmark>, Error> {
        let bookmarks = self
            .repo
            .find_many_bookmarks(BookmarksFindManyParams {
                profile_id: session.profile_id,
                limit: (PAGINATION_LIMIT + 1) as i64,
                published_at: params.published_at,
            })
            .await?;

        Ok(Paginated::<Bookmark> {
            has_more: bookmarks.len() > PAGINATION_LIMIT,
            data: bookmarks.into_iter().take(PAGINATION_LIMIT).collect(),
        })
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Bookmark, Error> {
        self.repo
            .find_one_bookmark(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn create(
        &self,
        mut data: CreateBookmark,
        session: Session,
    ) -> Result<Bookmark, Error> {
        let scraped = self.scraper.scrape(&mut data.url)?;

        self.repo
            .create_bookmark(BookmarksCreateData {
                url: data.url,
                bookmark: scraped,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateBookmark,
        session: Session,
    ) -> Result<Bookmark, Error> {
        self.repo
            .update_bookmark(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete_bookmark(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }
}

#[derive(Clone, Debug)]
pub struct BookmarksFindManyParams {
    pub profile_id: Uuid,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct BookmarksCreateData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct BookmarksUpdateData {
    pub tags: Option<Vec<Uuid>>,
}

impl From<UpdateBookmark> for BookmarksUpdateData {
    fn from(value: UpdateBookmark) -> Self {
        Self { tags: value.tags }
    }
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
