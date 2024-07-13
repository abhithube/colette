use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{Bookmark, Error};
use crate::common::FindOneParams;

#[async_trait]
pub trait BookmarksRepository {
    async fn find_many(&self, params: BookmarkFindManyParams) -> Result<Vec<Bookmark>, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct BookmarkFindManyParams {
    pub profile_id: String,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub should_filter: bool,
    pub collection_id: Option<String>,
}
