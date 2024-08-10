use tower_sessions::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};
pub use tower_sessions_sqlx_store::{PostgresStore, SqliteStore};

#[derive(Clone, Debug)]
pub enum SessionBackend {
    Postgres(PostgresStore),
    Sqlite(SqliteStore),
}

#[async_trait::async_trait]
impl SessionStore for SessionBackend {
    async fn save(&self, session_record: &Record) -> Result<(), session_store::Error> {
        match self {
            SessionBackend::Postgres(store) => store.save(session_record).await,
            SessionBackend::Sqlite(store) => store.save(session_record).await,
        }
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>, session_store::Error> {
        match self {
            SessionBackend::Postgres(store) => store.load(session_id).await,
            SessionBackend::Sqlite(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<(), session_store::Error> {
        match self {
            SessionBackend::Postgres(store) => store.delete(session_id).await,
            SessionBackend::Sqlite(store) => store.delete(session_id).await,
        }
    }
}

#[async_trait::async_trait]
impl ExpiredDeletion for SessionBackend {
    async fn delete_expired(&self) -> Result<(), session_store::Error> {
        match self {
            SessionBackend::Postgres(store) => store.delete_expired().await,
            SessionBackend::Sqlite(store) => store.delete_expired().await,
        }
    }
}
