#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("either feature \"postgres\" or feature \"sqlite\" must be enabled");

#[cfg(feature = "postgres")]
pub use colette_postgres::PostgresSessionRepository;
#[cfg(feature = "sqlite")]
pub use colette_sqlite::SqliteSessionRepository;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

#[derive(Clone, Debug)]
pub enum SessionBackend {
    #[cfg(feature = "postgres")]
    Postgres(colette_postgres::PostgresSessionRepository),
    #[cfg(feature = "sqlite")]
    Sqlite(colette_sqlite::SqliteSessionRepository),
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
