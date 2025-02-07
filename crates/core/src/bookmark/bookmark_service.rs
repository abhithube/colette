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
    storage::DynStorage,
};

const BASE_DIR: &str = "colette";

pub struct BookmarkService {
    repository: Box<dyn BookmarkRepository>,
    client: Box<dyn HttpClient>,
    object_store: Box<dyn ObjectStore>,
    archive_thumbnail_storage: Arc<Mutex<DynStorage<ArchiveThumbnailJob>>>,
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    bucket_url: String,
}

impl BookmarkService {
    pub fn new(
        repository: impl BookmarkRepository,
        http_client: impl HttpClient,
        object_store: impl ObjectStore,
        archive_thumbnail_storage: Arc<Mutex<DynStorage<ArchiveThumbnailJob>>>,
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
        bucket_url: String,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            client: Box::new(http_client),
            object_store: Box::new(object_store),
            archive_thumbnail_storage,
            plugins,
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
                url: data.url,
                title: data.title,
                thumbnail_url: data.thumbnail_url,
                published_at: data.published_at,
                author: data.author,
                folder_id: data.folder_id,
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
                    archived_url: None,
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
                        archived_url: bookmark.archived_url.clone(),
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
                archived_url: bookmark.archived_url,
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
                let (_, body) = self.client.get(&data.url).await?;
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
                let (_, body) = self.client.get(&data.url).await?;
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

                let (_, body) = self.client.get(&thumbnail_url).await?;

                let format = image::guess_format(&body)?;
                let extension = format.extensions_str()[0];

                let object_path = format!("{}/{}.{}", BASE_DIR, file_name, extension);

                self.object_store
                    .put(&Path::parse(&object_path).unwrap(), body.into())
                    .await?;

                let archived_url =
                    Url::parse(&format!("{}/{}", self.bucket_url, object_path)).unwrap();

                self.repository
                    .update(IdParams::new(bookmark_id, user_id), BookmarkUpdateData {
                        archived_url: Some(Some(archived_url)),
                        ..Default::default()
                    })
                    .await?;
            }
            ThumbnailOperation::Delete => {}
        }

        if let Some(archived_url) = data.archived_url {
            let object_path = archived_url.as_str().replace(&self.bucket_url, "");

            self.object_store
                .delete(&Path::parse(&object_path).unwrap())
                .await?;

            self.repository
                .update(IdParams::new(bookmark_id, user_id), BookmarkUpdateData {
                    archived_url: Some(None),
                    ..Default::default()
                })
                .await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkListQuery {
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BookmarkCreate {
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkUpdate {
    pub title: Option<Option<String>>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_url: None,
            folder_id: value.folder_id,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BookmarkScrape {
    pub url: Url,
}

#[derive(Clone, Debug)]
pub struct BookmarkScraped {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BookmarkPersist {
    pub url: Url,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct ThumbnailArchive {
    pub operation: ThumbnailOperation,
    pub archived_url: Option<Url>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ThumbnailOperation {
    Upload(Url),
    Delete,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrapeBookmarkJob {
    pub url: Url,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJob {
    pub operation: ThumbnailOperation,
    pub archived_url: Option<Url>,
    pub bookmark_id: Uuid,
    pub user_id: Uuid,
}
