use uuid::Uuid;

use super::{ApiKey, ApiKeyCursor, ApiKeyFindParams, ApiKeyRepository};
use crate::{
    Handler, RepositoryError,
    pagination::{Paginated, paginate},
};

#[derive(Debug, Clone)]
pub struct ListApiKeysQuery {
    pub cursor: Option<ApiKeyCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListApiKeysHandler {
    api_key_repository: Box<dyn ApiKeyRepository>,
}

impl ListApiKeysHandler {
    pub fn new(api_key_repository: impl ApiKeyRepository) -> Self {
        Self {
            api_key_repository: Box::new(api_key_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListApiKeysQuery> for ListApiKeysHandler {
    type Response = Paginated<ApiKey, ApiKeyCursor>;
    type Error = ListApiKeysError;

    async fn handle(&self, query: ListApiKeysQuery) -> Result<Self::Response, Self::Error> {
        let api_keys = self
            .api_key_repository
            .find(ApiKeyFindParams {
                user_id: Some(query.user_id),
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(api_keys, limit))
        } else {
            Ok(Paginated {
                items: api_keys,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListApiKeysError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
