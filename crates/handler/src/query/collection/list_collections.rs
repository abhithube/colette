use colette_core::{
    auth::UserId,
    collection::{CollectionCursor, CollectionDto, CollectionFindParams, CollectionRepository},
    common::RepositoryError,
    pagination::{Paginated, paginate},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ListCollectionsQuery {
    pub cursor: Option<CollectionCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListCollectionsHandler<CR: CollectionRepository> {
    collection_repository: CR,
}

impl<CR: CollectionRepository> ListCollectionsHandler<CR> {
    pub fn new(collection_repository: CR) -> Self {
        Self {
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<CR: CollectionRepository> Handler<ListCollectionsQuery> for ListCollectionsHandler<CR> {
    type Response = Paginated<CollectionDto, CollectionCursor>;
    type Error = ListCollectionsError;

    async fn handle(&self, query: ListCollectionsQuery) -> Result<Self::Response, Self::Error> {
        let collections = self
            .collection_repository
            .find(CollectionFindParams {
                user_id: query.user_id,
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                id: None,
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
