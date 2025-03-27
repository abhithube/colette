use std::collections::HashMap;

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_queue::JobProducer;
use colette_storage::StorageClient;
use colette_util::{base64, thumbnail};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Bookmark, BookmarkFilter, BookmarkScraper, Cursor, Error, ExtractedBookmark, ScraperError,
    bookmark_repository::{BookmarkParams, BookmarkRepository},
};
use crate::{
    collection::{CollectionParams, CollectionRepository},
    common::{PAGINATION_LIMIT, Paginated},
    job::{Job, JobRepository},
    tag::TagRepository,
};

const BOOKMARKS_DIR: &str = "bookmarks";

pub struct BookmarkService {
    bookmark_repository: Box<dyn BookmarkRepository>,
    tag_repository: Box<dyn TagRepository>,
    collection_repository: Box<dyn CollectionRepository>,
    job_repository: Box<dyn JobRepository>,
    http_client: Box<dyn HttpClient>,
    storage_client: Box<dyn StorageClient>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
}

impl BookmarkService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        tag_repository: impl TagRepository,
        collection_repository: impl CollectionRepository,
        job_repository: impl JobRepository,
        http_client: impl HttpClient,
        storage_client: impl StorageClient,
        archive_thumbnail_producer: impl JobProducer,
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            tag_repository: Box::new(tag_repository),
            collection_repository: Box::new(collection_repository),
            job_repository: Box::new(job_repository),
            http_client: Box::new(http_client),
            storage_client: Box::new(storage_client),
            archive_thumbnail_producer: Box::new(Mutex::new(archive_thumbnail_producer)),
            plugins,
        }
    }

    pub async fn list_bookmarks(
        &self,
        query: BookmarkListQuery,
        user_id: String,
    ) -> Result<Paginated<Bookmark>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let mut filter = Option::<BookmarkFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let mut collections = self
                .collection_repository
                .query(CollectionParams {
                    id: Some(collection_id),
                    user_id: Some(user_id.clone()),
                    ..Default::default()
                })
                .await?;
            if collections.is_empty() {
                return Ok(Paginated {
                    data: Default::default(),
                    cursor: None,
                });
            }

            filter = Some(collections.swap_remove(0).filter);
        }

        let mut bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                filter,
                tags: query.tags,
                user_id: Some(user_id),
                cursor,
                limit: Some(PAGINATION_LIMIT + 1),
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

    pub async fn get_bookmark(&self, id: Uuid, user_id: String) -> Result<Bookmark, Error> {
        let mut bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Err(Error::NotFound(id));
        }

        let bookmark = bookmarks.swap_remove(0);
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(bookmark.id));
        }

        Ok(bookmark)
    }

    pub async fn create_bookmark(
        &self,
        data: BookmarkCreate,
        user_id: String,
    ) -> Result<Bookmark, Error> {
        let builder = Bookmark::builder()
            .link(data.url)
            .title(data.title)
            .maybe_thumbnail_url(data.thumbnail_url)
            .maybe_published_at(data.published_at)
            .maybe_author(data.author)
            .user_id(user_id.clone());

        let bookmark = if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_by_ids(ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect::<Vec<_>>();

            builder.tags(tags).build()
        } else {
            builder.build()
        };

        self.bookmark_repository.save(&bookmark).await?;

        if let Some(thumbnail_url) = bookmark.thumbnail_url.clone() {
            let data = serde_json::to_value(&ArchiveThumbnailJobData {
                operation: ThumbnailOperation::Upload(thumbnail_url),
                archived_path: None,
                bookmark_id: bookmark.id,
            })?;

            let job = Job::builder()
                .job_type("archive_thumbnail".into())
                .data(data)
                .build();

            self.job_repository.save(&job).await?;

            let mut producer = self.archive_thumbnail_producer.lock().await;

            producer.push(job.id).await?;
        }

        Ok(bookmark)
    }

    pub async fn update_bookmark(
        &self,
        id: Uuid,
        data: BookmarkUpdate,
        user_id: String,
    ) -> Result<Bookmark, Error> {
        let Some(mut bookmark) = self.bookmark_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        let new_thumbnail = data.thumbnail_url.clone();

        if let Some(title) = data.title {
            bookmark.title = title;
        }
        if let Some(thumbnail_url) = data.thumbnail_url {
            bookmark.thumbnail_url = thumbnail_url;
        }
        if let Some(published_at) = data.published_at {
            bookmark.published_at = published_at;
        }
        if let Some(author) = data.author {
            bookmark.author = author;
        }

        if let Some(ids) = data.tags {
            let tags = self
                .tag_repository
                .find_by_ids(ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect();

            bookmark.tags = Some(tags);
        }

        bookmark.updated_at = Utc::now();
        self.bookmark_repository.save(&bookmark).await?;

        if let Some(thumbnail_url) = new_thumbnail {
            if thumbnail_url == bookmark.thumbnail_url {
                let data = serde_json::to_value(&ArchiveThumbnailJobData {
                    operation: if let Some(thumbnail_url) = thumbnail_url {
                        ThumbnailOperation::Upload(thumbnail_url)
                    } else {
                        ThumbnailOperation::Delete
                    },
                    archived_path: bookmark.archived_path.clone(),
                    bookmark_id: bookmark.id,
                })?;

                let job = Job::builder()
                    .job_type("archive_thumbnail".into())
                    .data(data)
                    .build();

                self.job_repository.save(&job).await?;

                let mut producer = self.archive_thumbnail_producer.lock().await;

                producer.push(job.id).await?;
            }
        }

        Ok(bookmark)
    }

    pub async fn delete_bookmark(&self, id: Uuid, user_id: String) -> Result<(), Error> {
        let Some(bookmark) = self.bookmark_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.bookmark_repository.delete_by_id(id).await?;

        let data = serde_json::to_value(&ArchiveThumbnailJobData {
            operation: ThumbnailOperation::Delete,
            archived_path: bookmark.archived_path,
            bookmark_id: bookmark.id,
        })?;

        let job = Job::builder()
            .job_type("archive_thumbnail".into())
            .data(data)
            .build();

        self.job_repository.save(&job).await?;

        let mut producer = self.archive_thumbnail_producer.lock().await;

        producer.push(job.id).await?;

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
                let body = self.http_client.get(&data.url).await?;
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

        let processed = match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(&mut data.url).await,
            None => {
                let body = self.http_client.get(&data.url).await?;
                let metadata =
                    colette_meta::parse_metadata(body.reader()).map_err(ScraperError::Parse)?;

                let bookmark = ExtractedBookmark::from(metadata);

                bookmark.try_into().map_err(ScraperError::Postprocess)
            }
        }?;

        let bookmark = Bookmark::builder()
            .link(data.url)
            .title(processed.title)
            .maybe_thumbnail_url(processed.thumbnail)
            .maybe_published_at(processed.published)
            .maybe_author(processed.author)
            .user_id(data.user_id)
            .build();

        self.bookmark_repository.upsert(&bookmark).await
    }

    pub async fn archive_thumbnail(
        &self,
        bookmark_id: Uuid,
        data: ThumbnailArchive,
    ) -> Result<(), Error> {
        match data.operation {
            ThumbnailOperation::Upload(thumbnail_url) => {
                let file_name = thumbnail::generate_filename(&thumbnail_url);

                let body = self.http_client.get(&thumbnail_url).await?;

                let format = image::guess_format(&body)?;
                let extension = format.extensions_str()[0];

                let object_path = format!("{}/{}.{}", BOOKMARKS_DIR, file_name, extension);

                self.storage_client
                    .upload(&object_path, body.into())
                    .await?;

                self.bookmark_repository
                    .set_archived_path(bookmark_id, Some(object_path))
                    .await?;
            }
            ThumbnailOperation::Delete => {}
        }

        if let Some(archived_path) = data.archived_path {
            self.storage_client.delete(&archived_path).await?;

            self.bookmark_repository
                .set_archived_path(bookmark_id, None)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkListQuery {
    pub collection_id: Option<Uuid>,
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
    pub user_id: String,
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
pub struct ScrapeBookmarkJobData {
    pub url: Url,
    pub user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJobData {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: Uuid,
}
