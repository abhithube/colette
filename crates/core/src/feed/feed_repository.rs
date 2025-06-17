use url::Url;
use uuid::Uuid;

use super::{Error, Feed};

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync + 'static {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error>;

    async fn find_by_source_url(&self, source_url: Url) -> Result<Option<Feed>, Error> {
        Ok(self
            .query(FeedParams {
                source_url: Some(source_url),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &mut Feed) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedParams {
    pub id: Option<Uuid>,
    pub source_url: Option<Url>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}
