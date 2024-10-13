use colette_core::cleanup::{CleanupRepository, Error};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;

pub struct SqliteCleanupRepository {
    pub(crate) pool: Pool,
}

impl SqliteCleanupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for SqliteCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let mut count = {
                let (sql, values) =
                    colette_sql::feed_entry::delete_many().build_rusqlite(SqliteQueryBuilder);

                tx.execute(&sql, &*values.as_params())?
            };
            if count > 0 {
                println!("Deleted {} orphaned feed entries", count);
            }

            count = {
                let (sql, values) =
                    colette_sql::feed::delete_many().build_rusqlite(SqliteQueryBuilder);

                tx.execute(&sql, &*values.as_params())?
            };
            if count > 0 {
                println!("Deleted {} orphaned feeds", count);
            }

            Ok::<_, rusqlite::Error>(())
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}
