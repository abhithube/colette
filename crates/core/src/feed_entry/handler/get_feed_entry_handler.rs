use crate::{
    Handler,
    common::RepositoryError,
    feed_entry::{FeedEntry, FeedEntryFindParams, FeedEntryId, FeedEntryRepository},
};

#[derive(Debug, Clone)]
pub struct GetFeedEntryQuery {
    pub id: FeedEntryId,
}

pub struct GetFeedEntryHandler<FER: FeedEntryRepository> {
    feed_entry_repository: FER,
}

impl<FER: FeedEntryRepository> GetFeedEntryHandler<FER> {
    pub fn new(feed_entry_repository: FER) -> Self {
        Self {
            feed_entry_repository,
        }
    }
}

#[async_trait::async_trait]
impl<FER: FeedEntryRepository> Handler<GetFeedEntryQuery> for GetFeedEntryHandler<FER> {
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
    NotFound(FeedEntryId),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
