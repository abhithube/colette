use chrono::{DateTime, Utc};
use torii_core::{Session, SessionStorage, UserId, error::StorageError, session::SessionToken};

use super::LibsqlBackend;

#[async_trait::async_trait]
impl SessionStorage for LibsqlBackend {
    type Error = torii_core::Error;

    async fn create_session(&self, session: &Session) -> Result<Session, Self::Error> {
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO sessions (token, user_id, user_agent, ip_address, created_at, updated_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
            )
            .await
            .map_err(|e| {
                StorageError::Database(e.to_string())
            })?;

        let row = stmt
            .query_row(libsql::params![
                session.token.as_str(),
                session.user_id.as_str(),
                session.user_agent.as_deref(),
                session.ip_address.as_deref(),
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.expires_at.to_rfc3339(),
            ])
            .await
            .map_err(|_| StorageError::Database("Failed to create session".to_string()))?;

        let row = libsql::de::from_row::<SessionRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(row.into())
    }

    async fn get_session(&self, token: &SessionToken) -> Result<Option<Session>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM sessions WHERE token = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(libsql::params![token.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get session".to_string()))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        let row = libsql::de::from_row::<SessionRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(Some(row.into()))
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<(), Self::Error> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM sessions WHERE token = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![token.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete session".to_string()))?;

        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<(), Self::Error> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM sessions WHERE expires_at < ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![Utc::now().to_rfc3339()])
            .await
            .map_err(|_| {
                StorageError::Database("Failed to cleanup expired sessions".to_string())
            })?;

        Ok(())
    }

    async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM sessions WHERE user_id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete user sessions".to_string()))?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct SessionRow {
    token: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    expires_at: DateTime<Utc>,
    user_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SessionRow> for Session {
    fn from(value: SessionRow) -> Self {
        Session::builder()
            .token(value.token.into())
            .user_agent(value.user_agent)
            .ip_address(value.ip_address)
            .expires_at(value.expires_at)
            .user_id(UserId::new(&value.user_id))
            .created_at(value.created_at)
            .updated_at(value.updated_at)
            .build()
            .unwrap()
    }
}
