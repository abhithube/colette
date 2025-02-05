pub use colette_scraper::feed::ProcessedFeed;
use futures::stream::BoxStream;
use uuid::Uuid;

use super::{Cursor, Error, Feed};
use crate::common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Updatable};

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
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error>;

    fn stream(&self) -> BoxStream<Result<String, Error>>;
}

#[derive(Clone, Debug, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedCacheData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug, Default)]
pub struct FeedUpdateData {
    pub title: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
}
