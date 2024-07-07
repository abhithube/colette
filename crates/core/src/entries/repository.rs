use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::Error;
use crate::Entry;

#[async_trait]
pub trait EntriesRepository {
    async fn find_many(&self, params: EntryFindManyParams) -> Result<Vec<Entry>, Error>;
}

pub struct EntryFindManyParams {
    pub profile_id: String,
    pub limit: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<String>,
    pub has_read: Option<bool>,
}
