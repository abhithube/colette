use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use super::{Error, Feed};

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync + 'static {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error>;

    async fn save(&self, data: &Feed) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedParams {
    pub id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
