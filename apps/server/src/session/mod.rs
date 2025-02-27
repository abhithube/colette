use ::redis::aio::MultiplexedConnection;
pub use redis::RedisStore;
pub use sqlite::SqliteStore;
use tower_sessions_core::{
    SessionStore,
    session::{Id, Record},
    session_store,
};

mod redis;
mod sqlite;

#[derive(Debug, Clone)]
pub enum AppSessionStore {
    Redis(RedisStore<MultiplexedConnection>),
    Sqlite(SqliteStore),
}

#[async_trait::async_trait]
impl SessionStore for AppSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        match self {
            AppSessionStore::Redis(store) => store.create(record).await,
            AppSessionStore::Sqlite(store) => store.create(record).await,
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        match self {
            AppSessionStore::Redis(store) => store.save(record).await,
            AppSessionStore::Sqlite(store) => store.save(record).await,
        }
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match self {
            AppSessionStore::Redis(store) => store.load(session_id).await,
            AppSessionStore::Sqlite(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        match self {
            AppSessionStore::Redis(store) => store.delete(session_id).await,
            AppSessionStore::Sqlite(store) => store.delete(session_id).await,
        }
    }
}
