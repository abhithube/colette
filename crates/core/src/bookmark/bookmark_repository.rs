use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Bookmark, Error};
use crate::{Tag, bookmark::BookmarkFilter};

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

    async fn import(&self, data: ImportBookmarksData) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkParams {
    pub id: Option<Uuid>,
    pub filter: Option<BookmarkFilter>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub with_tags: bool,
}

pub enum BookmarkUpsertType {
    Id,
    Link,
}

pub struct ImportBookmarksData {
    pub bookmarks: Vec<Bookmark>,
    pub tags: Vec<Tag>,
    pub user_id: Uuid,
}
