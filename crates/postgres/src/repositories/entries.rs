use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
    Entry,
};
use colette_database::profile_feed_entries::UpdateParams;
use sqlx::PgPool;

use crate::queries;

pub struct EntriesPostgresRepository {
    pool: PgPool,
}

impl EntriesPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EntriesRepository for EntriesPostgresRepository {
    async fn find_many(&self, params: EntriesFindManyParams) -> Result<Vec<Entry>, Error> {
        let entries = queries::profile_feed_entries::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(entries)
    }

    async fn update(&self, params: FindOneParams, data: EntriesUpdateData) -> Result<Entry, Error> {
        let entry = queries::profile_feed_entries::update(
            &self.pool,
            UpdateParams {
                id: params.id,
                profile_id: params.profile_id,
                has_read: data.has_read,
            },
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(entry)
    }
}
