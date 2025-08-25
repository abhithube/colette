use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_crud::BookmarkFilter;
use uuid::Uuid;

use crate::Cursor;

#[derive(Debug, Clone)]
pub struct CollectionDto {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionCursor {
    pub title: String,
}

impl Cursor for CollectionDto {
    type Data = CollectionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}

pub trait CollectionQueryRepository: Sync {
    fn query(
        &self,
        params: CollectionQueryParams,
    ) -> impl Future<Output = Result<Vec<CollectionDto>, RepositoryError>> + Send;

    fn query_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<CollectionDto>, RepositoryError>> + Send {
        async move {
            let mut collections = self
                .query(CollectionQueryParams {
                    user_id,
                    id: Some(id),
                    ..Default::default()
                })
                .await?;
            if collections.is_empty() {
                return Ok(None);
            }

            Ok(Some(collections.swap_remove(0)))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CollectionQueryParams {
    pub user_id: Uuid,
    pub id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}
