use std::time::SystemTime;

use deadpool_postgres::{tokio_postgres::error::SqlState, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

use super::query;

#[derive(Debug, Clone)]
pub struct PostgresSessionStore {
    pool: Pool,
}

impl PostgresSessionStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionStore for PostgresSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = SystemTime::from(record.expiry_date);

        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, _) = query::insert(record.id.to_string(), &payload, expires_at.into())
            .build_postgres(PostgresQueryBuilder);

        let stmt = tx
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        loop {
            match tx
                .execute(&stmt, &[&record.id.to_string(), &payload, &expires_at])
                .await
            {
                Ok(_) => break,
                Err(e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => {
                    record.id = Id::default();
                }
                Err(e) => {
                    return Err(session_store::Error::Backend(e.to_string()));
                }
            }
        }

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at: SystemTime = record.expiry_date.into();

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) = query::upsert(record.id.to_string(), &payload, expires_at.into())
            .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) =
            query::select_by_id(session_id.to_string()).build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let row = client
            .query_opt(&stmt, &values.as_params())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        if let Some(row) = row {
            let data = row.get::<_, Vec<u8>>(0);

            Ok(Some(serde_json::from_slice(&data).map_err(|e| {
                session_store::Error::Decode(e.to_string())
            })?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) =
            query::delete_by_id(session_id.to_string()).build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for PostgresSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) = query::delete_many().build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
