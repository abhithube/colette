use colette_common::RepositoryError;
use url::Url;

use crate::{Feed, FeedEntry, FeedId};

pub trait FeedRepository: Sync {
    fn find_by_id(
        &self,
        id: FeedId,
    ) -> impl Future<Output = Result<Option<Feed>, RepositoryError>> + Send;

    fn find_by_source_url(
        &self,
        source_url: &Url,
    ) -> impl Future<Output = Result<Option<Feed>, RepositoryError>> + Send;

    fn find_outdated(
        &self,
        params: FeedFindOutdatedParams,
    ) -> impl Future<Output = Result<Vec<Feed>, RepositoryError>> + Send;

    fn upsert(&self, data: FeedBatch) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn mark_as_failed(
        &self,
        source_url: Url,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<FeedId>,
    pub ready_to_refresh: bool,
    pub cursor: Option<Url>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FeedFindOutdatedParams {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FeedBatch {
    pub feed: Feed,
    pub feed_entries: Vec<FeedEntry>,
}
