use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use uuid::Uuid;

use crate::Cursor;

#[derive(Debug, Clone)]
pub struct PersonalAccessTokenDto {
    pub id: Uuid,
    pub title: String,
    pub preview: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for PersonalAccessTokenDto {
    type Data = PatCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}

pub trait PatQueryRepository: Sync {
    fn query(
        &self,
        params: PatQueryParams,
    ) -> impl Future<Output = Result<Vec<PersonalAccessTokenDto>, RepositoryError>> + Send;

    fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<PersonalAccessTokenDto>, RepositoryError>> + Send {
        async move {
            let mut pats = self
                .query(PatQueryParams {
                    id: Some(id),
                    user_id,
                    ..Default::default()
                })
                .await?;
            if pats.is_empty() {
                return Ok(None);
            }

            Ok(Some(pats.swap_remove(0)))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PatQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
