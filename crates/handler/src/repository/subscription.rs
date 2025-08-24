use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_core::pagination::Cursor;
use url::Url;
use uuid::Uuid;

use crate::TagDto;

#[derive(Debug, Clone)]
pub struct SubscriptionDto {
    pub id: Uuid,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    pub tags: Vec<TagDto>,
    pub unread_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionCursor {
    pub title: String,
    pub id: Uuid,
}

impl Cursor for SubscriptionDto {
    type Data = SubscriptionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
            id: self.id,
        }
    }
}

#[async_trait::async_trait]
pub trait SubscriptionQueryRepository: Send + Sync + 'static {
    async fn query(
        &self,
        params: SubscriptionQueryParams,
    ) -> Result<Vec<SubscriptionDto>, RepositoryError>;

    async fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<SubscriptionDto>, RepositoryError> {
        let mut subscriptions = self
            .query(SubscriptionQueryParams {
                user_id,
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscriptions.is_empty() {
            return Ok(None);
        }

        Ok(Some(subscriptions.swap_remove(0)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<(String, Uuid)>,
    pub limit: Option<usize>,
}
