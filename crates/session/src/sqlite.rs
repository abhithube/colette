use async_trait::async_trait;
use deadpool_sqlite::Pool;
use rusqlite::{params, OptionalExtension};
use time::OffsetDateTime;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Debug, thiserror::Error)]
pub enum SqliteStoreError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    Connection(#[from] deadpool_sqlite::InteractError),

    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),

    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),
}

impl From<SqliteStoreError> for session_store::Error {
    fn from(value: SqliteStoreError) -> Self {
        match value {
            SqliteStoreError::Encode(e) => session_store::Error::Encode(e.to_string()),
            SqliteStoreError::Decode(e) => session_store::Error::Decode(e.to_string()),
            _ => session_store::Error::Backend(value.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: Pool,
    table_name: String,
}

impl SqliteStore {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            table_name: "tower_sessions".into(),
        }
    }

    pub fn with_table_name(mut self, table_name: impl AsRef<str>) -> Result<Self, String> {
        let table_name = table_name.as_ref();
        if !is_valid_table_name(table_name) {
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
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
          create table if not exists {}
          (
              id text primary key not null,
              data blob not null,
              expiry_date integer not null
          )
          "#,
            self.table_name
        );

        conn.interact(move |conn| conn.execute(&query, ()))
            .await
            .map_err(SqliteStoreError::from)?
            .map_err(SqliteStoreError::from)?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SqliteStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
            delete from {table_name}
            where datetime(expiry_date) < datetime('now')
            "#,
            table_name = self.table_name
        );

        conn.interact(move |conn| conn.execute(&query, []))
            .await
            .map_err(SqliteStoreError::from)?
            .map_err(SqliteStoreError::from)?;

        Ok(())
    }
}

#[async_trait]
impl SessionStore for SqliteStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
          insert or abort into {table_name}
            (id, data, expiry_date) values (?, ?, ?)
          "#,
            table_name = self.table_name
        );

        let mut id = record.id;
        let payload = rmp_serde::to_vec(record).map_err(SqliteStoreError::Encode)?;
        let expiry = record.expiry_date;

        record.id = conn
            .interact(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(&query)?;

                    loop {
                        match stmt.execute(params![id.to_string(), payload, expiry]) {
                            Ok(_) => break,
                            Err(rusqlite::Error::SqliteFailure(e, _))
                                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
                            {
                                id = Id::default();
                            }
                            Err(e) => {
                                return Err(SqliteStoreError::Rusqlite(e));
                            }
                        };
                    }
                }

                tx.commit()?;

                Ok::<_, SqliteStoreError>(id)
            })
            .await
            .map_err(SqliteStoreError::from)?
            .map_err(SqliteStoreError::from)?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
      insert into {table_name}
      (id, data, expiry_date) values (?, ?, ?)
      on conflict(id) do update set
      data = excluded.data,
      expiry_date = excluded.expiry_date
      "#,
            table_name = self.table_name
        );

        let id = record.id.to_string();
        let payload = rmp_serde::to_vec(record).map_err(SqliteStoreError::Encode)?;
        let expiry = record.expiry_date;

        conn.interact(move |conn| {
            conn.prepare_cached(&query)?
                .execute(params![id, payload, expiry])
        })
        .await
        .map_err(SqliteStoreError::from)?
        .map_err(SqliteStoreError::from)?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
          select data from {}
          where id = ? and expiry_date > ?
          "#,
            self.table_name
        );

        let id = session_id.to_string();
        let date = OffsetDateTime::now_utc();

        let data = conn
            .interact(move |conn| {
                conn.prepare_cached(&query)?
                    .query_row(params![id, date], |row| row.get::<_, Vec<u8>>(0))
                    .optional()
            })
            .await
            .map_err(SqliteStoreError::from)?
            .map_err(SqliteStoreError::from)?;

        if let Some(data) = data {
            Ok(Some(
                rmp_serde::from_slice(&data).map_err(SqliteStoreError::Decode)?,
            ))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let conn = self.pool.get().await.map_err(SqliteStoreError::from)?;

        let query = format!(
            r#"
          delete from {} where id = ?
          "#,
            self.table_name
        );

        let id = session_id.to_string();

        conn.interact(move |conn| conn.prepare_cached(&query)?.execute(params![id]))
            .await
            .map_err(SqliteStoreError::from)?
            .map_err(SqliteStoreError::from)?;

        Ok(())
    }
}

fn is_valid_table_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
