#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("either feature \"postgres\" or feature \"sqlite\" must be enabled");

use tower_sessions::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};
#[cfg(feature = "postgres")]
pub use tower_sessions_sqlx_store::PostgresStore;
#[cfg(feature = "sqlite")]
pub use tower_sessions_sqlx_store::SqliteStore;

#[derive(Clone, Debug)]
pub enum SessionBackend {
    #[cfg(feature = "postgres")]
    Postgres(PostgresStore),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStore),
}

#[async_trait::async_trait]
impl SessionStore for SessionBackend {
    async fn save(&self, session_record: &Record) -> Result<(), session_store::Error> {
        match self {
            #[cfg(feature = "postgres")]
            SessionBackend::Postgres(store) => store.save(session_record).await,
            #[cfg(feature = "sqlite")]
            SessionBackend::Sqlite(store) => store.save(session_record).await,
        }
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>, session_store::Error> {
        match self {
            #[cfg(feature = "postgres")]
            SessionBackend::Postgres(store) => store.load(session_id).await,
            #[cfg(feature = "sqlite")]
            SessionBackend::Sqlite(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<(), session_store::Error> {
        match self {
            #[cfg(feature = "postgres")]
            SessionBackend::Postgres(store) => store.delete(session_id).await,
            #[cfg(feature = "sqlite")]
            SessionBackend::Sqlite(store) => store.delete(session_id).await,
        }
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for SessionBackend {
    async fn delete_expired(&self) -> Result<(), session_store::Error> {
        match self {
            #[cfg(feature = "postgres")]
            SessionBackend::Postgres(store) => store.delete_expired().await,
            #[cfg(feature = "sqlite")]
            SessionBackend::Sqlite(store) => store.delete_expired().await,
        }
    }
}
