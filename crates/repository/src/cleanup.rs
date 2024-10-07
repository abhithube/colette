use colette_core::cleanup::{CleanupRepository, Error};
use sqlx::PgPool;

pub struct CleanupSqlRepository {
    pub(crate) pool: PgPool,
}

impl CleanupSqlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for CleanupSqlRepository {
    async fn cleanup_feeds(&self) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut count = colette_postgres::feed_entry::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feed entries", count);
        }

        count = colette_postgres::feed::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feeds", count);
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        let count = colette_postgres::tag::delete_many(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned tags", count);
        }

        Ok(())
    }
}
