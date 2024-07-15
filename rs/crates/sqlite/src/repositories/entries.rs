use colette_core::{
    entries::{EntriesRepository, EntryFindManyParams, Error},
    Entry,
};
use sqlx::SqlitePool;

use crate::queries;

pub struct EntriesSqliteRepository {
    pool: SqlitePool,
}

impl EntriesSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EntriesRepository for EntriesSqliteRepository {
    async fn find_many(&self, params: EntryFindManyParams) -> Result<Vec<Entry>, Error> {
        let entries = queries::profile_feed_entries::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(entries)
    }
}
