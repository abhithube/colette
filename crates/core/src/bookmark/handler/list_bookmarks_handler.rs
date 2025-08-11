use crate::{
    Handler,
    bookmark::{Bookmark, BookmarkCursor, BookmarkFilter, BookmarkFindParams, BookmarkRepository},
    collection::{CollectionFindParams, CollectionId, CollectionRepository},
    common::RepositoryError,
    pagination::{Paginated, paginate},
    tag::TagId,
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct ListBookmarksQuery {
    pub collection_id: Option<CollectionId>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<BookmarkCursor>,
    pub limit: Option<usize>,
    pub with_tags: bool,
    pub user_id: UserId,
}

pub struct ListBookmarksHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    collection_repository: Box<dyn CollectionRepository>,
}

impl ListBookmarksHandler {
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        collection_repository: impl CollectionRepository,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListBookmarksQuery> for ListBookmarksHandler {
    type Response = Paginated<Bookmark, BookmarkCursor>;
    type Error = ListBookmarksError;

    async fn handle(&self, query: ListBookmarksQuery) -> Result<Self::Response, Self::Error> {
        let mut filter = Option::<BookmarkFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let mut collections = self
                .collection_repository
                .find(CollectionFindParams {
                    id: Some(collection_id),
                    user_id: Some(query.user_id),
                    ..Default::default()
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
                filter,
                tags: query.tags,
                user_id: Some(query.user_id),
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                with_tags: query.with_tags,
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
