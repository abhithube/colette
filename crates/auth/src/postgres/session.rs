use chrono::Utc;
use tokio_postgres::Row;
use torii_core::{Session, SessionStorage, UserId, error::StorageError, session::SessionToken};

use super::PostgresBackend;

#[async_trait::async_trait]
impl SessionStorage for PostgresBackend {
    type Error = torii_core::Error;

    async fn create_session(&self, session: &Session) -> Result<Session, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached(
                "INSERT INTO sessions (token, user_id, user_agent, ip_address, created_at, updated_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
            )
            .await
            .map_err(|e| {
                StorageError::Database(e.to_string())
            })?;

        let row = client
            .query_one(
                &stmt,
                &[
                    &session.token.as_str(),
                    &session.user_id.as_str(),
                    &session.user_agent.as_deref(),
                    &session.ip_address.as_deref(),
                    &session.created_at,
                    &session.updated_at,
                    &session.expires_at,
                ],
            )
            .await
            .map_err(|_| StorageError::Database("Failed to create session".to_string()))?;

        Ok(SessionRow(&row).into())
    }

    async fn get_session(&self, token: &SessionToken) -> Result<Option<Session>, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("SELECT * FROM sessions WHERE token = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = client
            .query_opt(&stmt, &[&token.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get session".to_string()))?;

        Ok(row.map(|e| SessionRow(&e).into()))
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<(), Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("DELETE FROM sessions WHERE token = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client
            .execute(&stmt, &[&token.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete session".to_string()))?;

        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<(), Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("DELETE FROM sessions WHERE expires_at < $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client.execute(&stmt, &[&Utc::now()]).await.map_err(|_| {
            StorageError::Database("Failed to cleanup expired sessions".to_string())
        })?;

        Ok(())
    }

    async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("DELETE FROM sessions WHERE user_id = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client
            .execute(&stmt, &[&user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete user sessions".to_string()))?;

        Ok(())
    }
}

struct SessionRow<'a>(&'a Row);

impl From<SessionRow<'_>> for Session {
    fn from(SessionRow(value): SessionRow<'_>) -> Self {
        Session::builder()
            .token(value.get::<_, String>("token").into())
            .user_agent(value.get("user_agent"))
            .ip_address(value.get("ip_address"))
            .expires_at(value.get("expires_at"))
            .user_id(value.get::<_, String>("user_id").into())
            .created_at(value.get("created_at"))
            .updated_at(value.get("updated_at"))
            .build()
            .unwrap()
    }
}
