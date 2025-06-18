use deadpool_postgres::Pool;
use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub struct PostgresMigrator<'a> {
    pool: &'a Pool,
}

impl<'a> PostgresMigrator<'a> {
    pub fn new(pool: &'a Pool) -> Self {
        Self { pool }
    }

    pub async fn is_fresh(&self, pool: &Pool) -> Result<bool, deadpool_postgres::PoolError> {
        let client = pool.get().await?;

        let row = client.query_one(r#"SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"#, &[]).await?;

        Ok(row.get::<_, i64>(0) == 0)
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for PostgresMigrator<'_> {
    type Error = deadpool_postgres::PoolError;

    async fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        let mut count = 0;
        for query in queries {
            tx.batch_execute(query).await?;

            count += 1;
        }

        tx.commit().await?;

        Ok(count)
    }
}

#[async_trait::async_trait]
impl AsyncQuery<Vec<Migration>> for PostgresMigrator<'_> {
    async fn query(
        &mut self,
        query: &str,
    ) -> Result<Vec<Migration>, <Self as AsyncTransaction>::Error> {
        let client = self.pool.get().await?;

        let mut migrations = Vec::<Migration>::new();

        let rows = client.query(query, &[]).await?;
        for row in rows {
            migrations.push(Migration::applied(
                row.get("version"),
                row.get("name"),
                OffsetDateTime::parse(row.get("applied_on"), &Rfc3339).unwrap(),
                row.get::<_, String>("checksum").parse().unwrap(),
            ));
        }

        Ok(migrations)
    }
}

impl AsyncMigrate for PostgresMigrator<'_> {}
