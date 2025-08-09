use uuid::Uuid;

use super::{FeedEntry, FeedEntryFindParams, FeedEntryRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetFeedEntryQuery {
    pub id: Uuid,
}

pub struct GetFeedEntryHandler {
    feed_entry_repository: Box<dyn FeedEntryRepository>,
}

impl GetFeedEntryHandler {
    pub fn new(feed_entry_repository: impl FeedEntryRepository) -> Self {
        Self {
            feed_entry_repository: Box::new(feed_entry_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetFeedEntryQuery> for GetFeedEntryHandler {
    type Response = FeedEntry;
    type Error = GetFeedEntryError;

    async fn handle(&self, query: GetFeedEntryQuery) -> Result<Self::Response, Self::Error> {
        let mut feed_entries = self
            .feed_entry_repository
            .find(FeedEntryFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if feed_entries.is_empty() {
            return Err(GetFeedEntryError::NotFound(query.id));
        }

        Ok(feed_entries.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetFeedEntryError {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
