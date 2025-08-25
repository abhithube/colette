use colette_common::RepositoryError;
use uuid::Uuid;

use crate::{Handler, Paginated, TagCursor, TagDto, TagQueryParams, TagQueryRepository, paginate};

#[derive(Debug, Clone)]
pub struct ListTagsQuery {
    pub cursor: Option<TagCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListTagsHandler<TQR: TagQueryRepository> {
    tag_query_repository: TQR,
}

impl<TQR: TagQueryRepository> ListTagsHandler<TQR> {
    pub fn new(tag_query_repository: TQR) -> Self {
        Self {
            tag_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<TQR: TagQueryRepository> Handler<ListTagsQuery> for ListTagsHandler<TQR> {
    type Response = Paginated<TagDto, TagCursor>;
    type Error = ListTagsError;

    async fn handle(&self, query: ListTagsQuery) -> Result<Self::Response, Self::Error> {
        let tags = self
            .tag_query_repository
            .query(TagQueryParams {
                user_id: query.user_id,
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(tags, limit))
        } else {
            Ok(Paginated {
                items: tags,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListTagsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
