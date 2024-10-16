use colette_core::cleanup::{CleanupRepository, Error, FeedCleanupInfo};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;

pub struct PostgresCleanupRepository {
    pool: Pool,
}

impl PostgresCleanupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for PostgresCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<FeedCleanupInfo, Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_count = {
            let (sql, values) =
                colette_sql::feed_entry::delete_many().build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        let feed_entry_count = {
            let (sql, values) =
                colette_sql::feed::delete_many().build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(FeedCleanupInfo {
            feed_count,
            feed_entry_count,
        })
    }
}
