use crate::{
    Handler,
    common::RepositoryError,
    feed::{Feed, FeedFindParams, FeedId, FeedRepository},
};

#[derive(Debug, Clone)]
pub struct GetFeedQuery {
    pub id: FeedId,
}

pub struct GetFeedHandler<FR: FeedRepository> {
    feed_repository: FR,
}

impl<FR: FeedRepository> GetFeedHandler<FR> {
    pub fn new(feed_repository: FR) -> Self {
        Self { feed_repository }
    }
}

#[async_trait::async_trait]
impl<FR: FeedRepository> Handler<GetFeedQuery> for GetFeedHandler<FR> {
    type Response = Feed;
    type Error = GetFeedError;

    async fn handle(&self, query: GetFeedQuery) -> Result<Self::Response, Self::Error> {
        let mut feeds = self
            .feed_repository
            .find(FeedFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(GetFeedError::NotFound(query.id));
        }

        Ok(feeds.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetFeedError {
    #[error("feed not found with ID: {0}")]
    NotFound(FeedId),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
