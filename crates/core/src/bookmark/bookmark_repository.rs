use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Bookmark, Error};
use crate::bookmark::BookmarkFilter;

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn query(&self, params: BookmarkParams) -> Result<Vec<Bookmark>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Bookmark>, Error> {
        Ok(self
            .query(BookmarkParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &Bookmark) -> Result<(), Error>;

    async fn upsert(&self, data: &Bookmark) -> Result<(), Error>;

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkParams {
    pub id: Option<Uuid>,
    pub filter: Option<BookmarkFilter>,
    pub user_id: Option<String>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}

pub enum BookmarkUpsertType {
    Id,
    Link,
}
