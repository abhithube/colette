use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use sqlx::{PgPool, Row};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub struct PostgresMigrator {
    pool: PgPool,
}

impl PostgresMigrator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn is_fresh(&self) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(r#"SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"#).fetch_one(&self.pool).await?;

        Ok(row.get::<i64, _>(0) == 0)
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for PostgresMigrator {
    type Error = sqlx::Error;

    async fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let mut tx = self.pool.begin().await?;

        let mut count = 0;
        for query in queries {
            sqlx::query(query).execute(&mut *tx).await?;

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
        let mut migrations = Vec::<Migration>::new();

        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
        for row in rows {
            migrations.push(Migration::applied(
                row.get("version"),
                row.get("name"),
                OffsetDateTime::parse(row.get("applied_on"), &Rfc3339).unwrap(),
                row.get::<String, _>("checksum").parse().unwrap(),
            ));
        }

        Ok(migrations)
    }
}

impl AsyncMigrate for PostgresMigrator {}
