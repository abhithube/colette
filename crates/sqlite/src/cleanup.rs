use colette_core::cleanup::{CleanupRepository, Error, FeedCleanupInfo};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteCleanupRepository {
    pool: SqlitePool,
}

impl SqliteCleanupRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for SqliteCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<FeedCleanupInfo, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_count = {
            let (sql, values) =
                colette_sql::feed_entry::delete_many().build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };

        let feed_entry_count = {
            let (sql, values) = colette_sql::feed::delete_many().build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(FeedCleanupInfo {
            feed_count,
            feed_entry_count,
        })
    }
}
