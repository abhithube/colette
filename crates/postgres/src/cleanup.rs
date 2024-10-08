use colette_core::cleanup::{CleanupRepository, Error};
use sqlx::PgPool;

use crate::query;

pub struct PostgresCleanupRepository {
    pub(crate) pool: PgPool,
}

impl PostgresCleanupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for PostgresCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut count = query::feed_entry::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feed entries", count);
        }

        count = query::feed::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feeds", count);
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        let count = query::tag::delete_many(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned tags", count);
        }

        Ok(())
    }
}
