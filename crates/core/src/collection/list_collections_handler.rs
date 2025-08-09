use uuid::Uuid;

use super::{Collection, CollectionCursor, CollectionFindParams, CollectionRepository};
use crate::{
    Handler, RepositoryError,
    pagination::{Paginated, paginate},
};

#[derive(Debug, Clone, Default)]
pub struct ListCollectionsQuery {
    pub cursor: Option<CollectionCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListCollectionsHandler {
    collection_repository: Box<dyn CollectionRepository>,
}

impl ListCollectionsHandler {
    pub fn new(collection_repository: impl CollectionRepository) -> Self {
        Self {
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListCollectionsQuery> for ListCollectionsHandler {
    type Response = Paginated<Collection, CollectionCursor>;
    type Error = ListCollectionsError;

    async fn handle(&self, query: ListCollectionsQuery) -> Result<Self::Response, Self::Error> {
        let collections = self
            .collection_repository
            .find(CollectionFindParams {
                user_id: Some(query.user_id),
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(collections, limit))
        } else {
            Ok(Paginated {
                items: collections,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListCollectionsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
