use chrono::{DateTime, Utc};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    session::{SessionDelete, SessionInsert, SessionSelect},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use torii_core::{Session, SessionStorage, UserId, error::StorageError, session::SessionToken};

use super::SqliteBackend;

#[async_trait::async_trait]
impl SessionStorage for SqliteBackend {
    type Error = torii_core::Error;

    async fn create_session(&self, session: &Session) -> Result<Session, Self::Error> {
        let (sql, values) = SessionInsert {
            token: session.token.as_str(),
            user_agent: session.user_agent.as_deref(),
            ip_address: session.ip_address.as_deref(),
            expires_at: session.expires_at,
            user_id: session.user_id.as_str(),
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, SessionRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                println!("{}", e);
                StorageError::Database("Failed to create session".to_string())
            })?;

        Ok(row.into())
    }

    async fn get_session(&self, token: &SessionToken) -> Result<Option<Session>, Self::Error> {
        let (sql, values) = SessionSelect {
            token: token.as_str(),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, SessionRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to get session".to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<(), Self::Error> {
        let (sql, values) = SessionDelete::Token(token.to_owned().into_inner())
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to delete session".to_string()))?;

        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<(), Self::Error> {
        let (sql, values) = SessionDelete::Expired
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|_| {
                StorageError::Database("Failed to cleanup expired sessions".to_string())
            })?;

        Ok(())
    }

    async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let (sql, values) = SessionDelete::UserId(user_id.to_owned().into_inner())
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|_| {
                StorageError::Database("Failed to delete sessions for user".to_string())
            })?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
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
    fn from(session: SessionRow) -> Self {
        Session::builder()
            .token(SessionToken::new(&session.token))
            .user_agent(session.user_agent)
            .ip_address(session.ip_address)
            .expires_at(session.expires_at)
            .user_id(UserId::new(&session.user_id))
            .created_at(session.created_at)
            .updated_at(session.updated_at)
            .build()
            .unwrap()
    }
}

impl From<Session> for SessionRow {
    fn from(session: Session) -> Self {
        SessionRow {
            token: session.token.into_inner(),
            user_id: session.user_id.into_inner(),
            user_agent: session.user_agent,
            ip_address: session.ip_address,
            created_at: session.created_at,
            updated_at: session.updated_at,
            expires_at: session.expires_at,
        }
    }
}
