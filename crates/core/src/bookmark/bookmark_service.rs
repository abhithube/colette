use std::sync::Arc;

use apalis_redis::{RedisContext, RedisError};
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_scraper::bookmark::BookmarkScraper;
use colette_util::{base64, thumbnail};
use object_store::{ObjectStore, path::Path};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Bookmark, Cursor, Error,
    bookmark_repository::{
        BookmarkCreateData, BookmarkFindParams, BookmarkRepository, BookmarkUpdateData,
    },
};
use crate::{
    common::{IdParams, NonEmptyString, PAGINATION_LIMIT, Paginated},
    storage::Storage,
};

const BASE_DIR: &str = "colette";

pub struct BookmarkService {
    repository: Box<dyn BookmarkRepository>,
    scraper: Arc<dyn BookmarkScraper>,
    http_client: Box<dyn HttpClient>,
    object_store: Box<dyn ObjectStore>,
    archive_thumbnail_storage: Arc<
        Mutex<dyn Storage<Job = ArchiveThumbnailJob, Context = RedisContext, Error = RedisError>>,
    >,
    bucket_url: String,
}

impl BookmarkService {
    pub fn new(
        repository: impl BookmarkRepository,
        scraper: Arc<dyn BookmarkScraper>,
        http_client: impl HttpClient,
        object_store: impl ObjectStore,
        archive_thumbnail_storage: Arc<
            Mutex<
                dyn Storage<Job = ArchiveThumbnailJob, Context = RedisContext, Error = RedisError>,
            >,
        >,
        bucket_url: String,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            scraper,
            http_client: Box::new(http_client),
            object_store: Box::new(object_store),
            archive_thumbnail_storage,
            bucket_url,
        }
    }

    pub async fn list_bookmarks(
        &self,
        query: BookmarkListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Bookmark>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut bookmarks = self
            .repository
            .find(BookmarkFindParams {
                folder_id: query.folder_id,
                tags: query.tags,
                user_id,
                limit: Some(PAGINATION_LIMIT as i64 + 1),
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
                let encoded = base64::encode(&c)?;

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
                title: data.title.into(),
                thumbnail_url: data.thumbnail_url.map(String::from),
                published_at: data.published_at,
                author: data.author.map(String::from),
                folder_id: data.folder_id,
                tags: data.tags.map(|e| e.into_iter().map(String::from).collect()),
                user_id,
            })
            .await?;

        let bookmark = self.get_bookmark(id, user_id).await?;

        if let (Some(thumbnail_url), None) = (&bookmark.thumbnail_url, &bookmark.archived_url) {
            let mut storage = self.archive_thumbnail_storage.lock().await;

            let url = thumbnail_url.parse().unwrap();
            storage
                .push(ArchiveThumbnailJob {
                    url,
                    bookmark_id: bookmark.id,
                    user_id,
                })
                .await
                .unwrap();
        }

        Ok(bookmark)
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

        let bookmark = self.get_bookmark(id, user_id).await?;

        if let (Some(thumbnail_url), None) = (&bookmark.thumbnail_url, &bookmark.archived_url) {
            let mut storage = self.archive_thumbnail_storage.lock().await;

            let url = thumbnail_url.parse().unwrap();
            storage
                .push(ArchiveThumbnailJob {
                    url,
                    bookmark_id: bookmark.id,
                    user_id,
                })
                .await
                .unwrap();
        }

        Ok(bookmark)
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

        Ok(scraped)
    }

    pub async fn archive_thumbnail(
        &self,
        bookmark_id: Uuid,
        data: ThumbnailArchive,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let file_name = thumbnail::generate_filename(&data.thumbnail_url);

        let (_, body) = self.http_client.get(&data.thumbnail_url).await?;

        let format = image::guess_format(&body)?;
        let extension = format.extensions_str()[0];

        let object_path = format!("{}/{}.{}", BASE_DIR, file_name, extension);

        self.object_store
            .put(&Path::parse(&object_path).unwrap(), body.into())
            .await?;

        let archived_url = Url::parse(&format!("{}/{}", self.bucket_url, object_path)).unwrap();

        self.repository
            .update(IdParams::new(bookmark_id, user_id), BookmarkUpdateData {
                archived_url: Some(Some(archived_url.to_string())),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkListQuery {
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BookmarkCreate {
    pub url: Url,
    pub title: NonEmptyString,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<NonEmptyString>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<NonEmptyString>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.map(String::from)),
            thumbnail_url: value.thumbnail_url.map(|e| e.map(String::from)),
            published_at: value.published_at,
            author: value.author.map(|e| e.map(String::from)),
            archived_url: None,
            folder_id: value.folder_id,
            tags: value
                .tags
                .map(|e| e.into_iter().map(String::from).collect()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BookmarkScrape {
    pub url: Url,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkScraped {
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ThumbnailArchive {
    pub thumbnail_url: Url,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScrapeBookmarkJob {
    pub url: Url,
    pub user_id: Uuid,
}

pub type ScrapeBookmarkStorage =
    Arc<Mutex<dyn Storage<Job = ScrapeBookmarkJob, Context = RedisContext, Error = RedisError>>>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJob {
    pub url: Url,
    pub bookmark_id: Uuid,
    pub user_id: Uuid,
}
