use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_netscape::{Item, Netscape};
use colette_queue::JobProducer;
use colette_scraper::bookmark::BookmarkScraper;
use colette_storage::StorageClient;
use colette_util::{hex_encode, sha256_hash};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Bookmark, BookmarkCursor, BookmarkFilter, Error, ImportBookmarksParams,
    bookmark_repository::{BookmarkFindParams, BookmarkRepository},
};
use crate::{
    bookmark::{
        BookmarkBatchItem, BookmarkInsertParams, BookmarkLinkTagParams, BookmarkUpdateParams,
    },
    collection::{CollectionFindParams, CollectionRepository},
    job::{JobInsertParams, JobRepository},
    pagination::{Paginated, paginate},
};

const THUMBNAILS_DIR: &str = "thumbnails";

pub struct BookmarkService {
    bookmark_repository: Arc<dyn BookmarkRepository>,
    collection_repository: Arc<dyn CollectionRepository>,
    job_repository: Arc<dyn JobRepository>,
    http_client: Box<dyn HttpClient>,
    scraper: BookmarkScraper,
    storage_client: Box<dyn StorageClient>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
    import_bookmarks_producer: Box<Mutex<dyn JobProducer>>,
}

impl BookmarkService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bookmark_repository: Arc<dyn BookmarkRepository>,
        collection_repository: Arc<dyn CollectionRepository>,
        job_repository: Arc<dyn JobRepository>,
        http_client: impl HttpClient,
        scraper: BookmarkScraper,
        storage_client: impl StorageClient,
        archive_thumbnail_producer: impl JobProducer,
        import_bookmarks_producer: impl JobProducer,
    ) -> Self {
        Self {
            bookmark_repository,
            collection_repository,
            job_repository,
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
        user_id: Uuid,
    ) -> Result<Paginated<Bookmark, BookmarkCursor>, Error> {
        let mut filter = Option::<BookmarkFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let mut collections = self
                .collection_repository
                .find(CollectionFindParams {
                    id: Some(collection_id),
                    user_id: Some(user_id),
                    ..Default::default()
                })
                .await?;
            if collections.is_empty() {
                return Ok(Paginated::default());
            }

            filter = Some(collections.swap_remove(0).filter);
        }

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                filter,
                tags: query.tags,
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(bookmarks, limit))
        } else {
            Ok(Paginated {
                items: bookmarks,
                ..Default::default()
            })
        }
    }

    pub async fn get_bookmark(
        &self,
        query: BookmarkGetQuery,
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let mut bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
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
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let id = self
            .bookmark_repository
            .insert(BookmarkInsertParams {
                link: data.url,
                title: data.title,
                thumbnail_url: data.thumbnail_url.clone(),
                published_at: data.published_at,
                author: data.author,
                user_id,
                upsert: false,
            })
            .await?;

        if let Some(thumbnail_url) = data.thumbnail_url {
            let data = serde_json::to_value(&ArchiveThumbnailJobData {
                operation: ThumbnailOperation::Upload(thumbnail_url),
                archived_path: None,
                bookmark_id: id,
            })?;

            let job_id = self
                .job_repository
                .insert(JobInsertParams {
                    job_type: "archive_thumbnail".into(),
                    data,
                    group_identifier: None,
                })
                .await?;

            let mut producer = self.archive_thumbnail_producer.lock().await;

            producer.push(job_id).await?;
        }

        self.get_bookmark(
            BookmarkGetQuery {
                id,
                with_tags: false,
            },
            user_id,
        )
        .await
    }

    pub async fn update_bookmark(
        &self,
        id: Uuid,
        data: BookmarkUpdate,
        user_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let Some(bookmark) = self.bookmark_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        let new_thumbnail = data.thumbnail_url.clone();

        self.bookmark_repository
            .update(BookmarkUpdateParams {
                id,
                title: data.title,
                thumbnail_url: data.thumbnail_url,
                published_at: data.published_at,
                author: data.author,
            })
            .await?;

        if let Some(thumbnail_url) = new_thumbnail
            && thumbnail_url == bookmark.thumbnail_url
        {
            let data = serde_json::to_value(&ArchiveThumbnailJobData {
                operation: if let Some(thumbnail_url) = thumbnail_url {
                    ThumbnailOperation::Upload(thumbnail_url)
                } else {
                    ThumbnailOperation::Delete
                },
                archived_path: bookmark.archived_path.clone(),
                bookmark_id: bookmark.id,
            })?;

            let job_id = self
                .job_repository
                .insert(JobInsertParams {
                    job_type: "archive_thumbnail".into(),
                    data,
                    group_identifier: None,
                })
                .await?;

            let mut producer = self.archive_thumbnail_producer.lock().await;

            producer.push(job_id).await?;
        }

        self.get_bookmark(
            BookmarkGetQuery {
                id,
                with_tags: false,
            },
            user_id,
        )
        .await
    }

    pub async fn delete_bookmark(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
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

        let job_id = self
            .job_repository
            .insert(JobInsertParams {
                job_type: "archive_thumbnail".into(),
                data,
                group_identifier: None,
            })
            .await?;

        let mut producer = self.archive_thumbnail_producer.lock().await;

        producer.push(job_id).await?;

        Ok(())
    }

    pub async fn link_bookmark_tags(
        &self,
        id: Uuid,
        data: LinkBookmarkTags,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let Some(bookmark) = self.bookmark_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if bookmark.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.bookmark_repository
            .link_tags(BookmarkLinkTagParams {
                bookmark_id: id,
                tag_ids: data.tag_ids,
            })
            .await?;

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

        self.bookmark_repository
            .insert(BookmarkInsertParams {
                link: data.url,
                title: processed.title,
                thumbnail_url: processed.thumbnail,
                published_at: processed.published,
                author: processed.author,
                user_id: data.user_id,
                upsert: true,
            })
            .await?;

        Ok(())
    }

    pub async fn archive_thumbnail(
        &self,
        bookmark_id: Uuid,
        data: ThumbnailArchive,
    ) -> Result<(), Error> {
        match data.operation {
            ThumbnailOperation::Upload(thumbnail_url) => {
                let file_name = format!(
                    "{}-{}",
                    Utc::now().timestamp(),
                    hex_encode(&sha256_hash(thumbnail_url.as_str())[..8])
                );

                let body = self.http_client.get(&thumbnail_url).await?;

                let format = image::guess_format(&body)?;
                let extension = format.extensions_str()[0];

                let object_path = format!("{THUMBNAILS_DIR}/{file_name}.{extension}");

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

    pub async fn import_bookmarks(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let netscape = colette_netscape::from_reader(raw.reader())?;

        let mut stack: Vec<(Option<String>, Item)> =
            netscape.items.into_iter().map(|e| (None, e)).collect();

        let mut tag_set = HashSet::<String>::new();
        let mut bookmark_map = HashMap::<Url, BookmarkBatchItem>::new();

        while let Some((parent_title, item)) = stack.pop() {
            if !item.item.is_empty() {
                for child in item.item {
                    stack.push((Some(item.title.clone()), child));
                }

                tag_set.insert(item.title);
            } else if let Some(link) = item.href {
                let link = link.parse::<Url>().unwrap();

                let bookmark =
                    bookmark_map
                        .entry(link.clone())
                        .or_insert_with(|| BookmarkBatchItem {
                            link,
                            title: item.title,
                            thumbnail_url: None,
                            published_at: None,
                            author: None,
                            created_at: item
                                .add_date
                                .and_then(|e| DateTime::<Utc>::from_timestamp(e, 0)),
                            updated_at: item
                                .last_modified
                                .and_then(|e| DateTime::<Utc>::from_timestamp(e, 0)),
                            tag_titles: Vec::new(),
                        });

                if let Some(title) = parent_title {
                    bookmark.tag_titles.push(title);
                }
            }
        }

        self.bookmark_repository
            .import(ImportBookmarksParams {
                bookmark_items: bookmark_map.into_values().collect(),
                tag_titles: tag_set.into_iter().collect(),
                user_id,
            })
            .await?;

        let data = serde_json::to_value(&ImportBookmarksJobData { user_id })?;

        let job_id = self
            .job_repository
            .insert(JobInsertParams {
                job_type: "import_bookmarks".into(),
                data,
                group_identifier: None,
            })
            .await?;

        let mut producer = self.import_bookmarks_producer.lock().await;

        producer.push(job_id).await?;

        Ok(())
    }

    pub async fn export_bookmarks(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let mut items = Vec::<Item>::new();
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id: Some(user_id),
                with_tags: true,
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

            if let Some(tags) = bookmark.tags
                && !tags.is_empty()
            {
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
            } else {
                items.push(item);
            }
        }

        items.append(&mut item_map.into_values().collect());

        let netscape = Netscape {
            items,
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
    pub cursor: Option<BookmarkCursor>,
    pub limit: Option<usize>,
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
pub struct LinkBookmarkTags {
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
pub struct ScrapeBookmarkJobData {
    pub url: Url,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJobData {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJobData {
    pub user_id: Uuid,
}
