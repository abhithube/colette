use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FeedEntryDto {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: Uuid,
}

#[async_trait::async_trait]
pub trait FeedEntryQueryRepository: Send + Sync + 'static {
    async fn query(
        &self,
        params: FeedEntryQueryParams,
    ) -> Result<Vec<FeedEntryDto>, RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedEntryQueryParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
