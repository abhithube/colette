use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use uuid::Uuid;

use crate::Cursor;

#[derive(Debug, Clone)]
pub struct TagDto {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TagCursor {
    pub title: String,
}

impl Cursor for TagDto {
    type Data = TagCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}

#[async_trait::async_trait]
pub trait TagQueryRepository: Sync {
    async fn query(&self, params: TagQueryParams) -> Result<Vec<TagDto>, RepositoryError>;

    async fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<TagDto>, RepositoryError> {
        let mut tags = self
            .query(TagQueryParams {
                user_id,
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Ok(None);
        }

        Ok(Some(tags.swap_remove(0)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct TagQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}
