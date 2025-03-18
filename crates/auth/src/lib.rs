pub use sqlite::SqliteBackend;
use torii_core::{
    NewUser, Session, SessionStorage, User, UserId, UserStorage, error::StorageError,
    session::SessionToken, storage::PasswordStorage,
};

mod sqlite;

#[derive(Clone)]
pub enum AuthAdapter {
    Sqlite(SqliteBackend),
}

#[async_trait::async_trait]
impl UserStorage for AuthAdapter {
    type Error = StorageError;

    async fn create_user(&self, user: &NewUser) -> Result<User, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.create_user(user).await,
        }
    }

    async fn get_user(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.get_user(id).await,
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.get_user_by_email(email).await,
        }
    }

    async fn get_or_create_user_by_email(&self, email: &str) -> Result<User, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.get_or_create_user_by_email(email).await,
        }
    }

    async fn update_user(&self, user: &User) -> Result<User, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.update_user(user).await,
        }
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.delete_user(id).await,
        }
    }

    async fn set_user_email_verified(&self, user_id: &UserId) -> Result<(), Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.set_user_email_verified(user_id).await,
        }
    }
}

#[async_trait::async_trait]
impl PasswordStorage for AuthAdapter {
    type Error = StorageError;

    async fn set_password_hash(
        &self,
        user_id: &UserId,
        hash: &str,
    ) -> Result<(), <Self as PasswordStorage>::Error> {
        match self {
            Self::Sqlite(backend) => backend.set_password_hash(user_id, hash).await,
        }
    }

    async fn get_password_hash(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, <Self as PasswordStorage>::Error> {
        match self {
            Self::Sqlite(backend) => backend.get_password_hash(user_id).await,
        }
    }
}

#[async_trait::async_trait]
impl SessionStorage for AuthAdapter {
    type Error = torii_core::Error;

    async fn create_session(&self, session: &Session) -> Result<Session, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.create_session(session).await,
        }
    }

    async fn get_session(&self, token: &SessionToken) -> Result<Option<Session>, Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.get_session(token).await,
        }
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<(), Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.delete_session(token).await,
        }
    }

    async fn cleanup_expired_sessions(&self) -> Result<(), Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.cleanup_expired_sessions().await,
        }
    }

    async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<(), Self::Error> {
        match self {
            Self::Sqlite(backend) => backend.delete_sessions_for_user(user_id).await,
        }
    }
}
