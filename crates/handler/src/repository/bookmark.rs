use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_crud::BookmarkFilter;
use url::Url;
use uuid::Uuid;

use crate::{Cursor, TagDto};

#[derive(Debug, Clone)]
pub struct BookmarkDto {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub archived_path: Option<String>,
    pub tags: Vec<TagDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BookmarkCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for BookmarkDto {
    type Data = BookmarkCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}

#[async_trait::async_trait]
pub trait BookmarkQueryRepository: Sync {
    async fn query(&self, params: BookmarkQueryParams)
    -> Result<Vec<BookmarkDto>, RepositoryError>;

    async fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BookmarkDto>, RepositoryError> {
        let mut bookmarks = self
            .query(BookmarkQueryParams {
                user_id,
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Ok(None);
        }

        Ok(Some(bookmarks.swap_remove(0)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub filter: Option<BookmarkFilter>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
