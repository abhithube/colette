use std::time::SystemTime;

use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Postgres};
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

use super::query;

#[derive(Debug, Clone)]
pub struct PostgresSessionStore {
    pool: Pool<Postgres>,
}

impl PostgresSessionStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionStore for PostgresSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = SystemTime::from(record.expiry_date);

        loop {
            let (sql, values) = query::insert(record.id.to_string(), &payload, expires_at.into())
                .build_sqlx(PostgresQueryBuilder);

            match sqlx::query_with(&sql, values).execute(&self.pool).await {
                Ok(_) => break,
                Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
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

        let (sql, values) = query::upsert(record.id.to_string(), &payload, expires_at.into())
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let (sql, values) =
            query::select_by_id(session_id.to_string()).build_sqlx(PostgresQueryBuilder);

        let data = sqlx::query_scalar_with::<_, Vec<u8>, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
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
        let (sql, values) =
            query::delete_by_id(session_id.to_string()).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for PostgresSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let (sql, values) = query::delete_many().build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
