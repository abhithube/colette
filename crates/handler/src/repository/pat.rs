use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_core::pagination::Cursor;
use uuid::Uuid;

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

#[async_trait::async_trait]
pub trait PatQueryRepository: Sync {
    async fn query(
        &self,
        params: PatQueryParams,
    ) -> Result<Vec<PersonalAccessTokenDto>, RepositoryError>;

    async fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<PersonalAccessTokenDto>, RepositoryError> {
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

#[derive(Debug, Clone, Default)]
pub struct PatQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
