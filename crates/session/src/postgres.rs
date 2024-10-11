use async_trait::async_trait;
use deadpool_postgres::Pool;
use time::OffsetDateTime;
use tokio_postgres::error::SqlState;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Debug, thiserror::Error)]
pub enum PostgresStoreError {
    #[error(transparent)]
    Postgres(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),

    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),
}

impl From<PostgresStoreError> for session_store::Error {
    fn from(value: PostgresStoreError) -> Self {
        match value {
            PostgresStoreError::Encode(e) => session_store::Error::Encode(e.to_string()),
            PostgresStoreError::Decode(e) => session_store::Error::Decode(e.to_string()),
            _ => session_store::Error::Backend(value.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresStore {
    pool: Pool,
    schema_name: String,
    table_name: String,
}

impl PostgresStore {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            schema_name: "tower_sessions".to_string(),
            table_name: "session".into(),
        }
    }

    pub fn with_schema_name(mut self, schema_name: impl AsRef<str>) -> Result<Self, String> {
        let schema_name = schema_name.as_ref();
        if !is_valid_identifier(schema_name) {
            return Err(format!(
                "Invalid schema name '{}'. Schema names must start with a letter or underscore \
                 (including letters with diacritical marks and non-Latin letters).Subsequent \
                 characters can be letters, underscores, digits (0-9), or dollar signs ($).",
                schema_name
            ));
        }

        schema_name.clone_into(&mut self.schema_name);

        Ok(self)
    }

    pub fn with_table_name(mut self, table_name: impl AsRef<str>) -> Result<Self, String> {
        let table_name = table_name.as_ref();
        if !is_valid_identifier(table_name) {
            return Err(format!(
                "Invalid table name '{}'. Table names must be alphanumeric and may contain \
               hyphens or underscores.",
                table_name
            ));
        }

        table_name.clone_into(&mut self.table_name);

        Ok(self)
    }

    pub async fn migrate(&self) -> session_store::Result<()> {
        let mut client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let tx = client
            .transaction()
            .await
            .map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"create schema if not exists "{schema_name}""#,
            schema_name = self.schema_name,
        );

        tx.execute(&query, &[])
            .await
            .map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
          create table if not exists "{schema_name}"."{table_name}"
            (
                id text primary key not null,
                data bytea not null,
                expiry_date timestamptz not null
            )
          "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        tx.execute(&query, &[])
            .await
            .map_err(PostgresStoreError::from)?;

        tx.commit().await.map_err(PostgresStoreError::from)?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for PostgresStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
            delete from "{schema_name}"."{table_name}"
            where datetime(expiry_date) < datetime('now')
            "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        client
            .execute(&query, &[])
            .await
            .map_err(PostgresStoreError::from)?;

        Ok(())
    }
}

#[async_trait]
impl SessionStore for PostgresStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let mut client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
          insert or abort into "{schema_name}"."{table_name}"
            (id, data, expiry_date) values ($1, $2, $3)
          "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        let payload = rmp_serde::to_vec(record).map_err(PostgresStoreError::from)?;
        let expiry = record.expiry_date;

        let tx = client
            .transaction()
            .await
            .map_err(PostgresStoreError::from)?;

        loop {
            match tx
                .execute(&query, &[&record.id.to_string(), &payload, &expiry])
                .await
            {
                Ok(_) => break,
                Err(e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => {
                    record.id = Id::default();
                }
                Err(e) => {
                    return Err(PostgresStoreError::from(e).into());
                }
            }
        }

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
      insert into "{schema_name}"."{table_name}"
      (id, data, expiry_date) values ($1, $2, $3)
      on conflict(id) do update set
      data = excluded.data,
      expiry_date = excluded.expiry_date
      "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        client
            .execute(
                &query,
                &[
                    &record.id.to_string(),
                    &rmp_serde::to_vec(record).map_err(PostgresStoreError::from)?,
                    &record.expiry_date,
                ],
            )
            .await
            .map_err(PostgresStoreError::from)?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
          select data from "{schema_name}"."{table_name}"
          where id = $1 and expiry_date > $2
          "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        let row = client
            .query_opt(
                &query,
                &[&session_id.to_string(), &OffsetDateTime::now_utc()],
            )
            .await
            .map_err(PostgresStoreError::from)?;

        if let Some(row) = row {
            let data = row.get::<_, Vec<u8>>(0);

            Ok(Some(
                rmp_serde::from_slice(&data).map_err(PostgresStoreError::from)?,
            ))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let client = self.pool.get().await.map_err(PostgresStoreError::from)?;

        let query = format!(
            r#"
          delete from "{schema_name}"."{table_name}" where id = $1
          "#,
            schema_name = self.schema_name,
            table_name = self.table_name
        );

        client
            .execute(&query, &[&session_id.to_string()])
            .await
            .map_err(PostgresStoreError::from)?;

        Ok(())
    }
}

fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
