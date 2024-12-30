use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_scraper::bookmark::{BookmarkScraper, ProcessedBookmark};
use colette_util::DataEncoder;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        Creatable, Deletable, Findable, IdParams, NonEmptyString, NonEmptyVec, Paginated,
        Updatable, PAGINATION_LIMIT,
    },
    Tag,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookmarkCreate {
    pub url: Url,
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkUpdate {
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkListQuery {
    pub tags: Option<Vec<String>>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookmarkScrape {
    pub url: Url,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkScraped {
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
}

pub struct BookmarkService {
    repository: Box<dyn BookmarkRepository>,
    scraper: Arc<dyn BookmarkScraper>,
    base64_encoder: Box<dyn DataEncoder<Cursor>>,
}

impl BookmarkService {
    pub fn new(
        repository: impl BookmarkRepository,
        scraper: Arc<dyn BookmarkScraper>,
        base64_encoder: impl DataEncoder<Cursor>,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            scraper,
            base64_encoder: Box::new(base64_encoder),
        }
    }

    pub async fn list_bookmarks(
        &self,
        query: BookmarkListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Bookmark>, Error> {
        let cursor = query
            .cursor
            .and_then(|e| self.base64_encoder.decode(&e).ok());

        let mut bookmarks = self
            .repository
            .find(BookmarkFindParams {
                tags: query.tags,
                user_id,
                limit: Some(PAGINATION_LIMIT + 1),
                cursor,
                ..Default::default()
            })
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if bookmarks.len() > limit {
            bookmarks = bookmarks.into_iter().take(limit).collect();

            if let Some(last) = bookmarks.last() {
                let c = Cursor {
                    created_at: last.created_at,
                };
                let encoded = self.base64_encoder.encode(&c)?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated {
            data: bookmarks,
            cursor,
        })
    }

    pub async fn get_bookmark(&self, id: Uuid, user_id: Uuid) -> Result<Bookmark, Error> {
        let mut bookmarks = self
            .repository
            .find(BookmarkFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(bookmarks.swap_remove(0))
    }

    pub async fn create_bookmark(
        &self,
        data: BookmarkCreate,
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let id = self
            .repository
            .create(BookmarkCreateData {
                url: data.url.into(),
                tags: data
                    .tags
                    .map(|e| Vec::from(e).into_iter().map(String::from).collect()),
                user_id,
            })
            .await?;

        self.get_bookmark(id, user_id).await
    }

    pub async fn update_bookmark(
        &self,
        id: Uuid,
        data: BookmarkUpdate,
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_bookmark(id, user_id).await
    }

    pub async fn delete_bookmark(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }

    pub async fn scrape_bookmark(
        &self,
        mut data: BookmarkScrape,
    ) -> Result<BookmarkScraped, Error> {
        let bookmark = self.scraper.scrape(&mut data.url).await?;

        let url = data.url.to_string();

        let scraped = BookmarkScraped {
            link: url.clone(),
            title: bookmark.title.clone(),
            thumbnail_url: bookmark.thumbnail.clone().map(String::from),
            published_at: bookmark.published,
            author: bookmark.author.clone(),
        };

        self.repository
            .cache(BookmarkCacheData { url, bookmark })
            .await?;

        Ok(scraped)
    }
}

#[async_trait::async_trait]
pub trait BookmarkRepository:
    Findable<Params = BookmarkFindParams, Output = Result<Vec<Bookmark>, Error>>
    + Creatable<Data = BookmarkCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = BookmarkUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error>;
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkFindParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkCreateData {
    pub url: String,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkUpdateData {
    pub tags: Option<Vec<String>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            tags: value
                .tags
                .map(|e| Vec::from(e).into_iter().map(String::from).collect()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BookmarkCacheData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error("bookmark not cached with URL: {0}")]
    Conflict(String),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
