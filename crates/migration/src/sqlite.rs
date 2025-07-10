use deadpool_sqlite::Pool;
use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub struct SqliteMigrator<'a> {
    pool: &'a Pool,
}

impl<'a> SqliteMigrator<'a> {
    pub fn new(pool: &'a Pool) -> Self {
        Self { pool }
    }

    pub async fn is_fresh(&self, pool: &Pool) -> Result<bool, deadpool_sqlite::PoolError> {
        let client = pool.get().await?;

        let count = client.interact(move |conn| {
            let count = conn.query_row(r#"SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"#, [], |e| e.get::<_, i64>(0))?;

            Ok::<_, deadpool_sqlite::PoolError>(count)
        }).await.unwrap()?;

        Ok(count == 0)
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for SqliteMigrator<'_> {
    type Error = deadpool_sqlite::PoolError;

    async fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let client = self.pool.get().await?;

        let queries = queries.iter().map(|e| e.to_string()).collect::<Vec<_>>();

        let count = client
            .interact(move |conn| {
                let tx = conn.transaction()?;

                let mut count = 0;
                for query in queries {
                    tx.execute_batch(&query)?;

                    count += 1;
                }

                tx.commit()?;

                Ok::<_, rusqlite::Error>(count)
            })
            .await
            .unwrap()?;

        Ok(count)
    }
}

#[async_trait::async_trait]
impl AsyncQuery<Vec<Migration>> for SqliteMigrator<'_> {
    async fn query(
        &mut self,
        query: &str,
    ) -> Result<Vec<Migration>, <Self as AsyncTransaction>::Error> {
        let client = self.pool.get().await?;

        let query = query.to_owned();

        let migrations = client
            .interact(move |conn| {
                let mut stmt = conn.prepare(&query)?;
                let mut rows = stmt.query([])?;

                let mut migrations = Vec::<Migration>::new();

                while let Some(row) = rows.next()? {
                    migrations.push(Migration::applied(
                        row.get_unwrap("version"),
                        row.get_unwrap("name"),
                        OffsetDateTime::parse(&row.get_unwrap::<_, String>("applied_on"), &Rfc3339)
                            .unwrap(),
                        row.get_unwrap::<_, String>("checksum").parse().unwrap(),
                    ));
                }

                Ok::<_, deadpool_sqlite::PoolError>(migrations)
            })
            .await
            .unwrap()?;

        Ok(migrations)
    }
}

impl AsyncMigrate for SqliteMigrator<'_> {}
