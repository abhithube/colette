use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_scraper::{bookmark::ProcessedBookmark, Scraper};
use colette_utils::DataEncoder;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable, PAGINATION_LIMIT},
    tag::TagCreate,
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
    pub sort_index: u32,
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookmarkCreate {
    pub url: Url,
    pub collection_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkUpdate {
    pub sort_index: Option<u32>,
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<TagCreate>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkListQuery {
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub sort_index: u32,
}

pub struct BookmarkService {
    repository: Arc<dyn BookmarkRepository>,
    scraper: Arc<dyn Scraper<ProcessedBookmark>>,
    base64_encoder: Arc<dyn DataEncoder<Cursor>>,
}

impl BookmarkService {
    pub fn new(
        repository: Arc<dyn BookmarkRepository>,
        scraper: Arc<dyn Scraper<ProcessedBookmark>>,
        base64_encoder: Arc<dyn DataEncoder<Cursor>>,
    ) -> Self {
        Self {
            repository,
            scraper,
            base64_encoder,
        }
    }

    pub async fn list_bookmarks(
        &self,
        query: BookmarkListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Bookmark>, Error> {
        let cursor = query
            .cursor
            .and_then(|e| self.base64_encoder.decode(&e).ok());

        let mut bookmarks = self
            .repository
            .list(
                profile_id,
                Some(PAGINATION_LIMIT),
                cursor,
                Some(BookmarkFindManyFilters {
                    collection_id: query.collection_id,
                    tags: query.tags,
                }),
            )
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if bookmarks.len() > limit {
            bookmarks = bookmarks.into_iter().take(limit).collect();

            if let Some(last) = bookmarks.last() {
                let c = Cursor {
                    sort_index: last.sort_index,
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

    pub async fn get_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<Bookmark, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_bookmark(
        &self,
        mut data: BookmarkCreate,
        profile_id: Uuid,
    ) -> Result<Bookmark, Error> {
        let scraped = self.scraper.scrape(&mut data.url).await?;

        self.repository
            .create(BookmarkCreateData {
                url: data.url.into(),
                bookmark: scraped,
                collection_id: data.collection_id,
                profile_id,
            })
            .await
    }

    pub async fn update_bookmark(
        &self,
        id: Uuid,
        data: BookmarkUpdate,
        profile_id: Uuid,
    ) -> Result<Bookmark, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }
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
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkFindManyFilters {
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkCreateData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub collection_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkUpdateData {
    pub sort_index: Option<u32>,
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            sort_index: value.sort_index,
            collection_id: value.collection_id,
            tags: value
                .tags
                .map(|e| e.into_iter().map(|e| e.title.into()).collect()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
