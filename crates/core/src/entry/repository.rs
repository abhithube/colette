use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    auth::UserId,
    common::RepositoryError,
    entry::{Entry, EntryDto, EntryFilter, EntryId},
    subscription::SubscriptionId,
    tag::TagId,
};

#[async_trait::async_trait]
pub trait EntryRepository: Send + Sync + 'static {
    async fn find(&self, params: EntryFindParams) -> Result<Vec<EntryDto>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: EntryId,
        user_id: UserId,
    ) -> Result<Option<Entry>, RepositoryError>;

    async fn save(&self, data: &Entry) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct EntryFindParams {
    pub user_id: UserId,
    pub id: Option<EntryId>,
    pub subscription_id: Option<SubscriptionId>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<TagId>>,
    pub filter: Option<EntryFilter>,
    pub cursor: Option<(DateTime<Utc>, Uuid)>,
    pub limit: Option<usize>,
}
