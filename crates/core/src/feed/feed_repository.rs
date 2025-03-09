use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use super::{Cursor, Error, Feed, ProcessedFeed};

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync + 'static {
    async fn find_feeds(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error>;

    async fn upsert_feed(&self, params: FeedUpsertParams) -> Result<Uuid, Error>;

    async fn stream_feed_urls(
        &self,
        params: FeedStreamUrlsParams,
    ) -> Result<BoxStream<Result<Url, Error>>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct FeedUpsertParams {
    pub url: Url,
    pub feed: ProcessedFeed,
}

pub struct FeedStreamUrlsParams;
