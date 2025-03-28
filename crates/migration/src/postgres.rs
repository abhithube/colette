use deadpool_postgres::Pool;
use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub struct PostgresMigrator {
    pool: Pool,
}

impl PostgresMigrator {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for PostgresMigrator {
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
impl AsyncQuery<Vec<Migration>> for PostgresMigrator {
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

impl AsyncMigrate for PostgresMigrator {}
