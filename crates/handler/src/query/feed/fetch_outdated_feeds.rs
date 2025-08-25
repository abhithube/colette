use colette_common::RepositoryError;
use colette_ingestion::{Feed, FeedFindOutdatedParams, FeedRepository};

use crate::Handler;

pub const FETCH_LIMIT: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct FetchOutdatedFeedsQuery {}

pub struct FetchOutdatedFeedsHandler<FR: FeedRepository> {
    feed_repository: FR,
}

impl<FR: FeedRepository> FetchOutdatedFeedsHandler<FR> {
    pub fn new(feed_repository: FR) -> Self {
        Self { feed_repository }
    }
}

#[async_trait::async_trait]
impl<FR: FeedRepository> Handler<FetchOutdatedFeedsQuery> for FetchOutdatedFeedsHandler<FR> {
    type Response = Vec<Feed>;
    type Error = ListFeedsError;

    async fn handle(&self, _query: FetchOutdatedFeedsQuery) -> Result<Self::Response, Self::Error> {
        let feeds = self
            .feed_repository
            .find_outdated(FeedFindOutdatedParams {
                limit: Some(FETCH_LIMIT),
            })
            .await?;

        Ok(feeds)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListFeedsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
