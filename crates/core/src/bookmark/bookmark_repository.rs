use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use super::{Bookmark, Cursor, Error, ProcessedBookmark};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[async_trait::async_trait]
pub trait BookmarkRepository:
    Findable<Params = BookmarkFindParams, Output = Result<Vec<Bookmark>, Error>>
    + Creatable<Data = BookmarkCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = BookmarkUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn save_scraped(&self, data: BookmarkScrapedData) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkFindParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct BookmarkCreateData {
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkUpdateData {
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub archived_path: Option<Option<String>>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct BookmarkScrapedData {
    pub url: Url,
    pub bookmark: ProcessedBookmark,
    pub user_id: Uuid,
}
