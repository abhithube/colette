use colette_core::{
    auth::UserId,
    common::RepositoryError,
    pagination::{Paginated, paginate},
    tag::{TagCursor, TagDto, TagFindParams, TagRepository},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ListTagsQuery {
    pub cursor: Option<TagCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListTagsHandler<TR: TagRepository> {
    tag_repository: TR,
}

impl<TR: TagRepository> ListTagsHandler<TR> {
    pub fn new(tag_repository: TR) -> Self {
        Self { tag_repository }
    }
}

#[async_trait::async_trait]
impl<TR: TagRepository> Handler<ListTagsQuery> for ListTagsHandler<TR> {
    type Response = Paginated<TagDto, TagCursor>;
    type Error = ListTagsError;

    async fn handle(&self, query: ListTagsQuery) -> Result<Self::Response, Self::Error> {
        let tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: query.user_id,
                id: None,
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
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
