use colette_common::RepositoryError;
use colette_core::pagination::{Paginated, paginate};
use uuid::Uuid;

use crate::{
    CollectionCursor, CollectionDto, CollectionQueryParams, CollectionQueryRepository, Handler,
};

#[derive(Debug, Clone)]
pub struct ListCollectionsQuery {
    pub cursor: Option<CollectionCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListCollectionsHandler<CQR: CollectionQueryRepository> {
    collection_query_repository: CQR,
}

impl<CQR: CollectionQueryRepository> ListCollectionsHandler<CQR> {
    pub fn new(collection_query_repository: CQR) -> Self {
        Self {
            collection_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<CQR: CollectionQueryRepository> Handler<ListCollectionsQuery> for ListCollectionsHandler<CQR> {
    type Response = Paginated<CollectionDto, CollectionCursor>;
    type Error = ListCollectionsError;

    async fn handle(&self, query: ListCollectionsQuery) -> Result<Self::Response, Self::Error> {
        let collections = self
            .collection_query_repository
            .query(CollectionQueryParams {
                user_id: query.user_id,
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
