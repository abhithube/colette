use colette_core::{
    common::RepositoryError,
    feed::{Feed, FeedCursor, FeedFindParams, FeedRepository},
    pagination::{Paginated, paginate},
};

use crate::Handler;

#[derive(Debug, Clone, Default)]
pub struct ListFeedsQuery {
    pub ready_to_refresh: bool,
    pub cursor: Option<FeedCursor>,
    pub limit: Option<usize>,
}

pub struct ListFeedsHandler<FR: FeedRepository> {
    feed_repository: FR,
}

impl<FR: FeedRepository> ListFeedsHandler<FR> {
    pub fn new(feed_repository: FR) -> Self {
        Self { feed_repository }
    }
}

#[async_trait::async_trait]
impl<FR: FeedRepository> Handler<ListFeedsQuery> for ListFeedsHandler<FR> {
    type Response = Paginated<Feed, FeedCursor>;
    type Error = ListFeedsError;

    async fn handle(&self, query: ListFeedsQuery) -> Result<Self::Response, Self::Error> {
        let feeds = self
            .feed_repository
            .find(FeedFindParams {
                ready_to_refresh: query.ready_to_refresh,
                cursor: query.cursor.map(|e| e.source_url),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(feeds, limit))
        } else {
            Ok(Paginated {
                items: feeds,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListFeedsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
