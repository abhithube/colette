use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Bookmark, Error};
use crate::bookmark::BookmarkFilter;

#[async_trait::async_trait]
pub trait BookmarkRepository: Send + Sync + 'static {
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Bookmark>, Error>;

    async fn save(&self, data: &Bookmark, upsert: Option<BookmarkUpsertType>) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindParams {
    pub filter: Option<BookmarkFilter>,
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<String>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}

pub enum BookmarkUpsertType {
    Id,
    Link,
}
