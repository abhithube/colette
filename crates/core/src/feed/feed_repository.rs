use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use super::{Cursor, Error, Feed, ProcessedFeed};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[async_trait::async_trait]
pub trait FeedRepository:
    Findable<Params = FeedFindParams, Output = Result<Vec<Feed>, Error>>
    + Creatable<Data = FeedCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = FeedUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error>;

    fn stream_urls(&self) -> BoxStream<Result<String, Error>>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct FeedCreateData {
    pub url: Url,
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct FeedUpdateData {
    pub title: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FeedScrapedData {
    pub url: Url,
    pub feed: ProcessedFeed,
    pub link_to_users: bool,
}
