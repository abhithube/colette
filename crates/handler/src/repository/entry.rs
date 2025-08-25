use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_crud::{EntryFilter, ReadStatus};
use url::Url;
use uuid::Uuid;

use crate::Cursor;

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

pub trait EntryQueryRepository: Sync {
    fn query(
        &self,
        params: EntryQueryParams,
    ) -> impl Future<Output = Result<Vec<EntryDto>, RepositoryError>> + Send;

    fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<EntryDto>, RepositoryError>> + Send {
        async move {
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
