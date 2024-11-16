use colette_core::cleanup::{CleanupRepository, Error, FeedCleanupInfo};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PostgresCleanupRepository {
    pool: PgPool,
}

impl PostgresCleanupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for PostgresCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<FeedCleanupInfo, Error> {
        let feed_count = {
            let (sql, values) =
                colette_sql::feed_entry::delete_many().build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };

        let feed_entry_count = {
            let (sql, values) = colette_sql::feed::delete_many().build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };

        Ok(FeedCleanupInfo {
            feed_count,
            feed_entry_count,
        })
    }
}
