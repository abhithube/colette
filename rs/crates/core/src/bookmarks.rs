use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{FindOneParams, Paginated, Session, PAGINATION_LIMIT};

#[derive(Debug)]
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
    pub collection_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ListBookmarksParams {
    pub published_at: Option<DateTime<Utc>>,
    pub collection_id: Option<Uuid>,
    pub is_default: Option<bool>,
}

#[async_trait::async_trait]
pub trait BookmarksRepository {
    async fn find_many(&self, params: BookmarkFindManyParams) -> Result<Vec<Bookmark>, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct BookmarksService {
    repo: Arc<dyn BookmarksRepository + Send + Sync>,
}

impl BookmarksService {
    pub fn new(repo: Arc<dyn BookmarksRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListBookmarksParams,
        session: Session,
    ) -> Result<Paginated<Bookmark>, Error> {
        let params = BookmarkFindManyParams {
            profile_id: session.profile_id,
            limit: (PAGINATION_LIMIT + 1) as i64,
            published_at: params.published_at,
            should_filter: params.collection_id.is_none() && params.is_default.is_some_and(|e| e),
            collection_id: params.collection_id,
        };
        let bookmarks = self.repo.find_many(params).await?;

        let paginated = Paginated::<Bookmark> {
            has_more: bookmarks.len() > PAGINATION_LIMIT,
            data: bookmarks.into_iter().take(PAGINATION_LIMIT).collect(),
        };

        Ok(paginated)
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}

pub struct BookmarkFindManyParams {
    pub profile_id: Uuid,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub should_filter: bool,
    pub collection_id: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
