use colette_core::{
    auth::UserId,
    bookmark::{
        BookmarkCursor, BookmarkDto, BookmarkFilter, BookmarkFindParams, BookmarkRepository,
    },
    collection::{CollectionFindParams, CollectionId, CollectionRepository},
    common::RepositoryError,
    pagination::{Paginated, paginate},
    tag::TagId,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ListBookmarksQuery {
    pub collection_id: Option<CollectionId>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<BookmarkCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListBookmarksHandler<BR: BookmarkRepository, CR: CollectionRepository> {
    bookmark_repository: BR,
    collection_repository: CR,
}

impl<BR: BookmarkRepository, CR: CollectionRepository> ListBookmarksHandler<BR, CR> {
    pub fn new(bookmark_repository: BR, collection_repository: CR) -> Self {
        Self {
            bookmark_repository,
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, CR: CollectionRepository> Handler<ListBookmarksQuery>
    for ListBookmarksHandler<BR, CR>
{
    type Response = Paginated<BookmarkDto, BookmarkCursor>;
    type Error = ListBookmarksError;

    async fn handle(&self, query: ListBookmarksQuery) -> Result<Self::Response, Self::Error> {
        let mut filter = Option::<BookmarkFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let mut collections = self
                .collection_repository
                .find(CollectionFindParams {
                    user_id: query.user_id,
                    id: Some(collection_id),
                    cursor: None,
                    limit: None,
                })
                .await?;
            if collections.is_empty() {
                return Ok(Paginated::default());
            }

            filter = Some(collections.swap_remove(0).filter);
        }

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id: query.user_id,
                filter,
                tags: query.tags,
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                id: None,
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
