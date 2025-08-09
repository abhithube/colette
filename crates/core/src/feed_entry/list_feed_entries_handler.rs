use uuid::Uuid;

use super::{FeedEntry, FeedEntryCursor, FeedEntryFindParams, FeedEntryRepository};
use crate::{
    Handler, RepositoryError,
    pagination::{Paginated, paginate},
};

#[derive(Debug, Clone, Default)]
pub struct ListFeedEntriesQuery {
    pub feed_id: Option<Uuid>,
    pub cursor: Option<FeedEntryCursor>,
    pub limit: Option<usize>,
}

pub struct ListFeedEntriesHandler {
    feed_entry_repository: Box<dyn FeedEntryRepository>,
}

impl ListFeedEntriesHandler {
    pub fn new(feed_entry_repository: impl FeedEntryRepository) -> Self {
        Self {
            feed_entry_repository: Box::new(feed_entry_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListFeedEntriesQuery> for ListFeedEntriesHandler {
    type Response = Paginated<FeedEntry, FeedEntryCursor>;
    type Error = ListFeedEntriesError;

    async fn handle(&self, query: ListFeedEntriesQuery) -> Result<Self::Response, Self::Error> {
        let feed_entries = self
            .feed_entry_repository
            .find(FeedEntryFindParams {
                feed_id: query.feed_id,
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(feed_entries, limit))
        } else {
            Ok(Paginated {
                items: feed_entries,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListFeedEntriesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
