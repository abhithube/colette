use std::time::SystemTime;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_sqlite::Pool;
use rusqlite::{params, OptionalExtension};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Debug, Clone)]
pub struct SqliteSessionRepository {
    pool: Pool,
}

impl SqliteSessionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for SqliteSessionRepository {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let mut id = record.id;
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = SystemTime::from(record.expiry_date);

        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, _) =
            colette_sql::session::insert(record.id.to_string(), &payload, expires_at.into())
                .build_rusqlite(SqliteQueryBuilder);

        record.id = conn
            .interact(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(&sql)?;

                    loop {
                        match stmt.execute(params![
                            id.to_string(),
                            payload,
                            DateTime::<Utc>::from(expires_at)
                        ]) {
                            Ok(_) => break,
                            Err(rusqlite::Error::SqliteFailure(e, _))
                                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
                            {
                                id = Id::default();
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        };
                    }
                }

                tx.commit()?;

                Ok::<_, rusqlite::Error>(id)
            })
            .await
            .unwrap()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at: SystemTime = record.expiry_date.into();

        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) =
            colette_sql::session::upsert(record.id.to_string(), &payload, expires_at.into())
                .build_rusqlite(SqliteQueryBuilder);

        conn.interact(move |conn| conn.prepare_cached(&sql)?.execute(&*values.as_params()))
            .await
            .unwrap()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) = colette_sql::session::select_by_id(session_id.to_string())
            .build_rusqlite(SqliteQueryBuilder);

        let data = conn
            .interact(move |conn| {
                conn.prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, Vec<u8>>(0))
                    .optional()
            })
            .await
            .unwrap()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        if let Some(data) = data {
            Ok(Some(serde_json::from_slice(&data).map_err(|e| {
                session_store::Error::Decode(e.to_string())
            })?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) = colette_sql::session::delete_by_id(session_id.to_string())
            .build_rusqlite(SqliteQueryBuilder);

        conn.interact(move |conn| conn.prepare_cached(&sql)?.execute(&*values.as_params()))
            .await
            .unwrap()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SqliteSessionRepository {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) = colette_sql::session::delete_many().build_rusqlite(SqliteQueryBuilder);

        conn.interact(move |conn| conn.execute(&sql, &*values.as_params()))
            .await
            .unwrap()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
