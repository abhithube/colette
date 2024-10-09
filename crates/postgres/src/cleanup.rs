use colette_core::cleanup::{CleanupRepository, Error};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;

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

        let mut result = {
            let (sql, values) =
                colette_sql::feed_entry::delete_many().build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if result.rows_affected() > 0 {
            println!("Deleted {} orphaned feed entries", result.rows_affected());
        }

        result = {
            let (sql, values) = colette_sql::feed::delete_many().build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if result.rows_affected() > 0 {
            println!("Deleted {} orphaned feeds", result.rows_affected());
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        let result = {
            let (sql, values) = colette_sql::tag::delete_many().build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if result.rows_affected() > 0 {
            println!("Deleted {} orphaned tags", result.rows_affected());
        }

        Ok(())
    }
}
