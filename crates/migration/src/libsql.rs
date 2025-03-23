use libsql::{Connection, de};
use refinery_core::{
    AsyncMigrate, Migration,
    traits::r#async::{AsyncQuery, AsyncTransaction},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub struct LibsqlMigrator {
    conn: Connection,
}

impl LibsqlMigrator {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl AsyncTransaction for LibsqlMigrator {
    type Error = libsql::Error;

    async fn execute(&mut self, queries: &[&str]) -> Result<usize, Self::Error> {
        let tx = self.conn.transaction().await?;

        let mut count = 0;
        for query in queries {
            tx.execute_batch(query).await?;

            count += 1;
        }

        tx.commit().await?;

        Ok(count)
    }
}

#[async_trait::async_trait]
impl AsyncQuery<Vec<Migration>> for LibsqlMigrator {
    async fn query(
        &mut self,
        query: &str,
    ) -> Result<Vec<Migration>, <Self as AsyncTransaction>::Error> {
        let mut migrations = Vec::<Migration>::new();

        let mut rows = self.conn.query(query, ()).await?;
        while let Some(row) = rows.next().await? {
            let row = de::from_row::<MigrationRow>(&row).unwrap();

            migrations.push(Migration::applied(
                row.version,
                row.name,
                OffsetDateTime::parse(&row.applied_on, &Rfc3339).unwrap(),
                row.checksum.parse().unwrap(),
            ));
        }

        Ok(migrations)
    }
}

impl AsyncMigrate for LibsqlMigrator {}

#[derive(Debug, serde::Deserialize)]
struct MigrationRow {
    version: i32,
    name: String,
    applied_on: String,
    checksum: String,
}
