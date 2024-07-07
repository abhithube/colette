use super::{EntriesRepository, EntryFindManyParams, Error, ListEntriesParams};
use crate::{
    common::{Paginated, Session, PAGINATION_LIMIT},
    Entry,
};

pub struct EntriesService {
    repo: Box<dyn EntriesRepository + Send + Sync>,
}

impl EntriesService {
    pub fn new(repo: Box<dyn EntriesRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListEntriesParams,
        session: Session,
    ) -> Result<Paginated<Entry>, Error> {
        let params = EntryFindManyParams {
            profile_id: session.profile_id,
            limit: PAGINATION_LIMIT + 1,
            published_at: params.published_at,
            feed_id: params.feed_id,
            has_read: params.has_read,
        };
        let entries = self.repo.find_many(params).await?;

        let limit = PAGINATION_LIMIT as usize;
        let paginated = Paginated::<Entry> {
            has_more: entries.len() > limit,
            data: entries.into_iter().take(limit).collect(),
        };

        Ok(paginated)
    }
}
