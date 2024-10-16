use std::time::SystemTime;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::error::SqlState;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Debug, Clone)]
pub struct PostgresSessionRepository {
    pool: Pool,
}

impl PostgresSessionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PostgresSessionRepository {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry = SystemTime::from(record.expiry_date);

        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, _) = colette_sql::session::insert(record.id.to_string(), &payload, expiry.into())
            .build_postgres(PostgresQueryBuilder);

        let tx = client
            .transaction()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let stmt = tx
            .prepare_cached(&sql)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        loop {
            match tx
                .execute(&stmt, &[&record.id.to_string(), &payload, &expiry])
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
        let expiry: SystemTime = record.expiry_date.into();

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) =
            colette_sql::session::upsert(record.id.to_string(), &payload, expiry.into())
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

        let (sql, values) = colette_sql::session::select_by_id(session_id.to_string())
            .build_postgres(PostgresQueryBuilder);

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

        let (sql, values) = colette_sql::session::delete_by_id(session_id.to_string())
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
}

#[async_trait]
impl ExpiredDeletion for PostgresSessionRepository {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let (sql, values) =
            colette_sql::session::delete_many().build_postgres(PostgresQueryBuilder);

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
