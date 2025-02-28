use std::{collections::HashMap, sync::Arc};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_util::{base64, thumbnail};
use object_store::{ObjectStore, path::Path};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Bookmark, BookmarkScrapedData, BookmarkScraper, Cursor, Error, ExtractedBookmark, ScraperError,
    bookmark_repository::{
        BookmarkCreateData, BookmarkFindParams, BookmarkRepository, BookmarkUpdateData,
    },
};
use crate::{
    common::{IdParams, PAGINATION_LIMIT, Paginated},
    worker::Storage,
};

const BOOKMARKS_DIR: &str = "bookmarks";

pub struct BookmarkService {
    repository: Box<dyn BookmarkRepository>,
    client: Box<dyn HttpClient>,
    object_store: Box<dyn ObjectStore>,
    archive_thumbnail_storage: Arc<Mutex<dyn Storage<ArchiveThumbnailJob>>>,
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
}

impl BookmarkService {
    pub fn new(
        repository: impl BookmarkRepository,
        http_client: impl HttpClient,
        object_store: impl ObjectStore,
        archive_thumbnail_storage: Arc<Mutex<dyn Storage<ArchiveThumbnailJob>>>,
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            client: Box::new(http_client),
            object_store: Box::new(object_store),
            archive_thumbnail_storage,
            plugins,
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
                    created_at: last.created_at.unwrap(),
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
                url: data.url,
                title: data.title,
                thumbnail_url: data.thumbnail_url,
                published_at: data.published_at,
                author: data.author,
                tags: data.tags,
                user_id,
            })
            .await?;

        let bookmark = self.get_bookmark(id, user_id).await?;

        if let Some(thumbnail_url) = bookmark.thumbnail_url.clone() {
            let mut storage = self.archive_thumbnail_storage.lock().await;

            storage
                .push(ArchiveThumbnailJob {
                    operation: ThumbnailOperation::Upload(thumbnail_url),
                    archived_path: None,
                    bookmark_id: bookmark.id,
                    user_id,
                })
                .await?;
        }

        Ok(bookmark)
    }

    pub async fn update_bookmark(
        &self,
        id: Uuid,
        data: BookmarkUpdate,
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let thumbnail_url = data.thumbnail_url.clone();

        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        let bookmark = self.get_bookmark(id, user_id).await?;

        if let Some(thumbnail_url) = thumbnail_url {
            if thumbnail_url == bookmark.thumbnail_url {
                let mut storage = self.archive_thumbnail_storage.lock().await;

                storage
                    .push(ArchiveThumbnailJob {
                        operation: if let Some(thumbnail_url) = thumbnail_url {
                            ThumbnailOperation::Upload(thumbnail_url)
                        } else {
                            ThumbnailOperation::Delete
                        },
                        archived_path: bookmark.archived_path.clone(),
                        bookmark_id: bookmark.id,
                        user_id,
                    })
                    .await?;
            }
        }

        Ok(bookmark)
    }

    pub async fn delete_bookmark(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let bookmark = self.get_bookmark(id, user_id).await?;

        self.repository.delete(IdParams::new(id, user_id)).await?;

        let mut storage = self.archive_thumbnail_storage.lock().await;

        storage
            .push(ArchiveThumbnailJob {
                operation: ThumbnailOperation::Delete,
                archived_path: bookmark.archived_path,
                bookmark_id: bookmark.id,
                user_id,
            })
            .await?;

        Ok(())
    }

    pub async fn scrape_bookmark(
        &self,
        mut data: BookmarkScrape,
    ) -> Result<BookmarkScraped, Error> {
        let host = data.url.host_str().unwrap();

        let bookmark = match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(&mut data.url).await,
            None => {
                let body = self.client.get(&data.url).await?;
                let metadata =
                    colette_meta::parse_metadata(body.reader()).map_err(ScraperError::Parse)?;

                let bookmark = ExtractedBookmark::from(metadata);

                bookmark.try_into().map_err(ScraperError::Postprocess)
            }
        }?;

        let scraped = BookmarkScraped {
            link: data.url,
            title: bookmark.title,
            thumbnail_url: bookmark.thumbnail,
            published_at: bookmark.published,
            author: bookmark.author,
        };

        Ok(scraped)
    }

    pub async fn scrape_and_persist_bookmark(
        &self,
        mut data: BookmarkPersist,
    ) -> Result<(), Error> {
        let host = data.url.host_str().unwrap();

        let bookmark = match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(&mut data.url).await,
            None => {
                let body = self.client.get(&data.url).await?;
                let metadata =
                    colette_meta::parse_metadata(body.reader()).map_err(ScraperError::Parse)?;

                let bookmark = ExtractedBookmark::from(metadata);

                bookmark.try_into().map_err(ScraperError::Postprocess)
            }
        }?;

        self.repository
            .save_scraped(BookmarkScrapedData {
                url: data.url,
                bookmark,
                user_id: data.user_id,
            })
            .await
    }

    pub async fn archive_thumbnail(
        &self,
        bookmark_id: Uuid,
        data: ThumbnailArchive,
        user_id: Uuid,
    ) -> Result<(), Error> {
        match data.operation {
            ThumbnailOperation::Upload(thumbnail_url) => {
                let file_name = thumbnail::generate_filename(&thumbnail_url);

                let body = self.client.get(&thumbnail_url).await?;

                let format = image::guess_format(&body)?;
                let extension = format.extensions_str()[0];

                let object_path = format!("{}/{}.{}", BOOKMARKS_DIR, file_name, extension);

                self.object_store
                    .put(&Path::parse(&object_path).unwrap(), body.into())
                    .await?;

                self.repository
                    .update(
                        IdParams::new(bookmark_id, user_id),
                        BookmarkUpdateData {
                            archived_path: Some(Some(object_path)),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            ThumbnailOperation::Delete => {}
        }

        if let Some(archived_path) = data.archived_path {
            self.object_store
                .delete(&Path::parse(&archived_path).unwrap())
                .await?;

            self.repository
                .update(
                    IdParams::new(bookmark_id, user_id),
                    BookmarkUpdateData {
                        archived_path: Some(None),
                        ..Default::default()
                    },
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkListQuery {
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookmarkCreate {
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkUpdate {
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub tags: Option<Vec<Uuid>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_path: None,
            tags: value.tags,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkScrape {
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct BookmarkScraped {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookmarkPersist {
    pub url: Url,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ThumbnailArchive {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThumbnailOperation {
    Upload(Url),
    Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeBookmarkJob {
    pub url: Url,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJob {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: Uuid,
    pub user_id: Uuid,
}
