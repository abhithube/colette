use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Bookmark, Cursor, Error};
use crate::common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Updatable};

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
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkFindParams {
    pub id: Option<Uuid>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkCreateData {
    pub url: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarkUpdateData {
    pub title: Option<Option<String>>,
    pub thumbnail_url: Option<Option<String>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub archived_url: Option<Option<String>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}
