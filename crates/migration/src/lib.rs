use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use sqlx::{Pool, Row, Sqlite, types::time::OffsetDateTime};
use time::format_description::well_known::Rfc3339;

pub struct SqliteMigrator {
    pool: Pool<Sqlite>,
}

impl SqliteMigrator {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for SqliteMigrator {
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
impl AsyncQuery<Vec<Migration>> for SqliteMigrator {
    async fn query(
        &mut self,
        query: &str,
    ) -> Result<Vec<Migration>, <Self as AsyncTransaction>::Error> {
        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
        let migrations = rows
            .into_iter()
            .map(|e| {
                Migration::applied(
                    e.get(0),
                    e.get(1),
                    OffsetDateTime::parse(&e.get::<String, _>(2), &Rfc3339).unwrap(),
                    e.get::<String, _>(3).parse().unwrap(),
                )
            })
            .collect();

        Ok(migrations)
    }
}

impl AsyncMigrate for SqliteMigrator {}
