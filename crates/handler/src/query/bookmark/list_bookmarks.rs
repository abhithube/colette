use colette_common::RepositoryError;
use colette_core::pagination::{Paginated, paginate};
use colette_crud::BookmarkFilter;
use uuid::Uuid;

use crate::{
    BookmarkCursor, BookmarkDto, BookmarkQueryParams, BookmarkQueryRepository,
    CollectionQueryRepository, Handler,
};

#[derive(Debug, Clone)]
pub struct ListBookmarksQuery {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<BookmarkCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListBookmarksHandler<BQR: BookmarkQueryRepository, CQR: CollectionQueryRepository> {
    bookmark_query_repository: BQR,
    collection_query_repository: CQR,
}

impl<BQR: BookmarkQueryRepository, CQR: CollectionQueryRepository> ListBookmarksHandler<BQR, CQR> {
    pub fn new(bookmark_query_repository: BQR, collection_query_repository: CQR) -> Self {
        Self {
            bookmark_query_repository,
            collection_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BQR: BookmarkQueryRepository, CQR: CollectionQueryRepository> Handler<ListBookmarksQuery>
    for ListBookmarksHandler<BQR, CQR>
{
    type Response = Paginated<BookmarkDto, BookmarkCursor>;
    type Error = ListBookmarksError;

    async fn handle(&self, query: ListBookmarksQuery) -> Result<Self::Response, Self::Error> {
        let mut filter = Option::<BookmarkFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let Some(collection) = self
                .collection_query_repository
                .query_by_id(collection_id, query.user_id)
                .await?
            else {
                return Ok(Paginated::default());
            };

            filter = Some(collection.filter);
        }

        let bookmarks = self
            .bookmark_query_repository
            .query(BookmarkQueryParams {
                user_id: query.user_id,
                filter,
                tags: query.tags,
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(bookmarks, limit))
        } else {
            Ok(Paginated {
                items: bookmarks,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListBookmarksError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
