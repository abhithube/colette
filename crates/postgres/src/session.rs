use std::time::SystemTime;

use async_trait::async_trait;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{PgPool, Row};
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Debug, Clone)]
pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PostgresSessionRepository {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry = SystemTime::from(record.expiry_date);

        loop {
            let (sql, values) =
                colette_sql::session::insert(record.id.to_string(), &payload, expiry.into())
                    .build_sqlx(PostgresQueryBuilder);

            match sqlx::query_with(&sql, values).execute(&self.pool).await {
                Ok(_) => break,
                Err(e) => match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        record.id = Id::default();
                    }
                    _ => {
                        return Err(session_store::Error::Backend(e.to_string()));
                    }
                },
            }
        }

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry: SystemTime = record.expiry_date.into();

        let (sql, values) =
            colette_sql::session::upsert(record.id.to_string(), &payload, expiry.into())
                .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let (sql, values) = colette_sql::session::select_by_id(session_id.to_string())
            .build_sqlx(PostgresQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        if let Some(row) = row {
            let data = row.get::<Vec<u8>, _>(0);

            Ok(Some(serde_json::from_slice(&data).map_err(|e| {
                session_store::Error::Decode(e.to_string())
            })?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let (sql, values) = colette_sql::session::delete_by_id(session_id.to_string())
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for PostgresSessionRepository {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let (sql, values) = colette_sql::session::delete_many().build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
