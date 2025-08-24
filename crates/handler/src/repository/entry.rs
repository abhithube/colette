use chrono::{DateTime, Utc};
use colette_core::{
    common::RepositoryError,
    entry::{EntryFilter, ReadStatus},
    pagination::Cursor,
};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EntryDto {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub read_status: ReadStatus,
    pub feed_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor for EntryDto {
    type Data = EntryCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            published_at: self.published_at,
            id: self.id,
        }
    }
}

#[async_trait::async_trait]
pub trait EntryQueryRepository: Send + Sync + 'static {
    async fn query(&self, params: EntryQueryParams) -> Result<Vec<EntryDto>, RepositoryError>;

    async fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<EntryDto>, RepositoryError> {
        let mut entries = self
            .query(EntryQueryParams {
                user_id,
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if entries.is_empty() {
            return Ok(None);
        }

        Ok(Some(entries.swap_remove(0)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntryQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub filter: Option<EntryFilter>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
