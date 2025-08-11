use crate::{
    Handler, RepositoryError,
    feed::{Feed, FeedFindParams, FeedId, FeedRepository},
};

#[derive(Debug, Clone)]
pub struct GetFeedQuery {
    pub id: FeedId,
}

pub struct GetFeedHandler {
    feed_repository: Box<dyn FeedRepository>,
}

impl GetFeedHandler {
    pub fn new(feed_repository: impl FeedRepository) -> Self {
        Self {
            feed_repository: Box::new(feed_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetFeedQuery> for GetFeedHandler {
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
