use colette_core::cleanup::{CleanupRepository, Error};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;

pub struct PostgresCleanupRepository {
    pub(crate) pool: Pool,
}

impl PostgresCleanupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for PostgresCleanupRepository {
    async fn cleanup_feeds(&self) -> Result<(), Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut count = {
            let (sql, values) =
                colette_sql::feed_entry::delete_many().build_postgres(PostgresQueryBuilder);

            tx.execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count > 0 {
            println!("Deleted {} orphaned feed entries", count);
        }

        count = {
            let (sql, values) =
                colette_sql::feed::delete_many().build_postgres(PostgresQueryBuilder);

            tx.execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count > 0 {
            println!("Deleted {} orphaned feeds", count);
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = {
            let (sql, values) =
                colette_sql::tag::delete_many().build_postgres(PostgresQueryBuilder);

            client
                .execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count > 0 {
            println!("Deleted {} orphaned tags", count);
        }

        Ok(())
    }
}
