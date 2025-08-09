use uuid::Uuid;

use super::{Tag, TagFindParams, TagRepository};
use crate::{
    Handler, RepositoryError,
    pagination::{Paginated, paginate},
    tag::TagCursor,
};

#[derive(Debug, Clone)]
pub struct ListTagsQuery {
    pub cursor: Option<TagCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListTagsHandler {
    tag_repository: Box<dyn TagRepository>,
}

impl ListTagsHandler {
    pub fn new(tag_repository: impl TagRepository) -> Self {
        Self {
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListTagsQuery> for ListTagsHandler {
    type Response = Paginated<Tag, TagCursor>;
    type Error = ListTagsError;

    async fn handle(&self, query: ListTagsQuery) -> Result<Self::Response, Self::Error> {
        let tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: Some(query.user_id),
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
