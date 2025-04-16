use std::collections::HashMap;

use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_netscape::{Item, Netscape};
use colette_queue::JobProducer;
use colette_scraper::bookmark::BookmarkScraper;
use colette_storage::StorageClient;
use colette_util::{base64, thumbnail};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Bookmark, BookmarkFilter, Cursor, Error, ImportBookmarksData,
    bookmark_repository::{BookmarkParams, BookmarkRepository},
};
use crate::{
    collection::{CollectionParams, CollectionRepository},
    common::{PAGINATION_LIMIT, Paginated},
    job::{Job, JobRepository},
    tag::TagRepository,
};

const THUMBNAILS_DIR: &str = "thumbnails";

pub struct BookmarkService {
    bookmark_repository: Box<dyn BookmarkRepository>,
    tag_repository: Box<dyn TagRepository>,
    collection_repository: Box<dyn CollectionRepository>,
    job_repository: Box<dyn JobRepository>,
    http_client: Box<dyn HttpClient>,
    scraper: BookmarkScraper,
    storage_client: Box<dyn StorageClient>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
    import_bookmarks_producer: Box<Mutex<dyn JobProducer>>,
}

impl BookmarkService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        tag_repository: impl TagRepository,
        collection_repository: impl CollectionRepository,
        job_repository: impl JobRepository,
        http_client: impl HttpClient,
        scraper: BookmarkScraper,
        storage_client: impl StorageClient,
        archive_thumbnail_producer: impl JobProducer,
        import_bookmarks_producer: impl JobProducer,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            tag_repository: Box::new(tag_repository),
            collection_repository: Box::new(collection_repository),
            job_repository: Box::new(job_repository),
            http_client: Box::new(http_client),
            scraper,
            storage_client: Box::new(storage_client),
            archive_thumbnail_producer: Box::new(Mutex::new(archive_thumbnail_producer)),
            import_bookmarks_producer: Box::new(Mutex::new(import_bookmarks_producer)),
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
                with_tags: query.with_tags,
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

    pub async fn get_bookmark(
        &self,
        query: BookmarkGetQuery,
        user_id: String,
    ) -> Result<Bookmark, Error> {
        let mut bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                id: Some(query.id),
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Err(Error::NotFound(query.id));
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
        let bookmark = Bookmark::builder()
            .link(data.url)
            .title(data.title)
            .maybe_thumbnail_url(data.thumbnail_url)
            .maybe_published_at(data.published_at)
            .maybe_author(data.author)
            .user_id(user_id.clone())
            .build();

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

    pub async fn link_bookmark_tags(
        &self,
        id: Uuid,
        data: LinkSubscriptionTags,
        user_id: String,
    ) -> Result<(), Error> {
        let Some(mut bookmark) = self.bookmark_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        let tags = if data.tag_ids.is_empty() {
            Vec::new()
        } else {
            self.tag_repository
                .find_by_ids(data.tag_ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect()
        };

        bookmark.tags = Some(tags);

        self.bookmark_repository.save(&bookmark).await?;

        Ok(())
    }

    pub async fn scrape_bookmark(
        &self,
        mut data: BookmarkScrape,
    ) -> Result<BookmarkScraped, Error> {
        let processed = self.scraper.scrape(&mut data.url).await?;

        let scraped = BookmarkScraped {
            link: data.url,
            title: processed.title,
            thumbnail_url: processed.thumbnail,
            published_at: processed.published,
            author: processed.author,
        };

        Ok(scraped)
    }

    pub async fn refresh_bookmark(&self, mut data: BookmarkRefresh) -> Result<(), Error> {
        let processed = self.scraper.scrape(&mut data.url).await?;

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

                let object_path = format!("{}/{}.{}", THUMBNAILS_DIR, file_name, extension);

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

    pub async fn import_bookmarks(&self, raw: Bytes, user_id: String) -> Result<(), Error> {
        let netscape = colette_netscape::from_reader(raw.reader())?;

        self.bookmark_repository
            .import(ImportBookmarksData {
                items: netscape.items,
                user_id: user_id.clone(),
            })
            .await?;

        let data = serde_json::to_value(&ImportBookmarksJobData { user_id })?;

        let job = Job::builder()
            .job_type("import_bookmarks".into())
            .data(data)
            .build();

        self.job_repository.save(&job).await?;

        let mut producer = self.import_bookmarks_producer.lock().await;

        producer.push(job.id).await?;

        Ok(())
    }

    pub async fn export_bookmarks(&self, user_id: String) -> Result<Bytes, Error> {
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                add_date: Some(bookmark.created_at.timestamp()),
                last_modified: Some(bookmark.updated_at.timestamp()),
                href: Some(bookmark.link.into()),
                ..Default::default()
            };

            if let Some(tags) = bookmark.tags {
                for tag in tags {
                    item_map
                        .entry(tag.id)
                        .or_insert_with(|| Item {
                            title: tag.title,
                            ..Default::default()
                        })
                        .item
                        .push(item.clone());
                }
            }
        }

        let netscape = Netscape {
            items: item_map.into_values().collect(),
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkListQuery {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<String>,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkGetQuery {
    pub id: Uuid,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct BookmarkCreate {
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkUpdate {
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct LinkSubscriptionTags {
    pub tag_ids: Vec<Uuid>,
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
pub struct BookmarkRefresh {
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJobData {
    pub user_id: String,
}
