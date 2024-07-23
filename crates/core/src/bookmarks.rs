use std::sync::Arc;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{FindOneParams, Paginated, Session, PAGINATION_LIMIT},
    utils::scraper::{self, ExtractorQuery, Scraper},
};

#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub custom_title: Option<String>,
    pub custom_thumbnail_url: Option<String>,
    pub custom_published_at: Option<DateTime<Utc>>,
    pub custom_author: Option<String>,
    pub collection_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateBookmark {
    pub url: String,
    pub collection_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct UpdateBookmark {
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ListBookmarksParams {
    pub published_at: Option<DateTime<Utc>>,
    pub collection_id: Option<Uuid>,
    pub is_default: Option<bool>,
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

#[async_trait::async_trait]
pub trait BookmarksRepository: Send + Sync {
    async fn find_many(&self, params: BookmarksFindManyParams) -> Result<Vec<Bookmark>, Error>;

    async fn create(&self, data: BookmarksCreateData) -> Result<Bookmark, Error>;

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
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
            .find_many(BookmarksFindManyParams {
                profile_id: session.profile_id,
                limit: (PAGINATION_LIMIT + 1) as i64,
                published_at: params.published_at,
                should_filter: params.collection_id.is_none()
                    && params.is_default.is_some_and(|e| e),
                collection_id: params.collection_id,
            })
            .await?;

        let paginated = Paginated::<Bookmark> {
            has_more: bookmarks.len() > PAGINATION_LIMIT,
            data: bookmarks.into_iter().take(PAGINATION_LIMIT).collect(),
        };

        Ok(paginated)
    }

    pub async fn create(
        &self,
        mut data: CreateBookmark,
        session: Session,
    ) -> Result<Bookmark, Error> {
        let scraped = self.scraper.scrape(&mut data.url)?;

        let bookmark = self
            .repo
            .create(BookmarksCreateData {
                url: data.url,
                bookmark: scraped,
                collection_id: data.collection_id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(bookmark)
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateBookmark,
        session: Session,
    ) -> Result<Bookmark, Error> {
        let bookmark = self
            .repo
            .update(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await?;

        Ok(bookmark)
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BookmarksFindManyParams {
    pub profile_id: Uuid,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub should_filter: bool,
    pub collection_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct BookmarksCreateData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub collection_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct BookmarksUpdateData {
    pub custom_title: Option<String>,
    pub custom_thumbnail_url: Option<String>,
    pub custom_published_at: Option<DateTime<Utc>>,
    pub custom_author: Option<String>,
}

impl From<UpdateBookmark> for BookmarksUpdateData {
    fn from(value: UpdateBookmark) -> Self {
        Self {
            custom_title: value.title,
            custom_thumbnail_url: value.thumbnail_url,
            custom_published_at: value.published_at,
            custom_author: value.author,
        }
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
