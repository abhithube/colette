use colette_core::cleanup::{CleanupRepository, Error, FeedCleanupInfo};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;

pub struct SqliteCleanupRepository {
    pool: Pool,
}

impl SqliteCleanupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for SqliteCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<FeedCleanupInfo, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let feed_count = {
                let (sql, values) =
                    colette_sql::feed_entry::delete_many().build_rusqlite(SqliteQueryBuilder);

                tx.execute(&sql, &*values.as_params())?
            };

            let feed_entry_count = {
                let (sql, values) =
                    colette_sql::feed::delete_many().build_rusqlite(SqliteQueryBuilder);

                tx.execute(&sql, &*values.as_params())?
            };

            Ok::<_, rusqlite::Error>(FeedCleanupInfo {
                feed_count: feed_count as u64,
                feed_entry_count: feed_entry_count as u64,
            })
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}
